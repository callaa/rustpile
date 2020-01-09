#!/usr/bin/env python3

from jinja2 import Template

from protogen import load_protocol_definition

template = Template("""
// Message definitions generated with protogen-rust.py

use super::serialization::{MessageReader, MessageWriter, DeserializationError};
use super::textmessage::TextMessage;
use std::convert::TryInto;
use std::fmt;

pub static VERSION: &'static str = "{{ version }}";

{# ### STRUCTS FOR MESSAGES WITH NONTRIVIAL PAYLOADS ### #}
{% for message in messages %}{% if message.fields|length > 1 %}

{# THE STRUCT FIELD TYPE IS USED FOR DAB DATA ONLY (AT THE MOMENT) #}
{% if message.fields[-1].subfields %}{% set sfield = message.fields[-1] %}
#[derive(Debug, PartialEq)]
pub struct {{ sfield.struct_name }} {
    {% for f in sfield.subfields %}
    pub {{ f.name }}: {{ field_rust_type(f) }},
    {% endfor %}
}
{% endif %}

#[derive(Debug, PartialEq)]
pub struct {{ message.name }}Message {
    {% for f in message.fields %}
    pub {{ f.name }}: {{ field_rust_type(f) }},
    {% endfor %}
}

impl {{ message.name }}Message {
    {# ADD MAX STRUCT ITEM COUNT CONSTANT #}
    {% if message.fields[-1].max_items %}
    pub const MAX_ITEMS: usize = {{ message.fields[-1].max_items }};
    {% endif %}

    {# ADD FLAG CONSTANTS #}
    {% for field in message.fields %}{% if field.flags %}
    {% for flag in field.flags %}
    pub const {{ field.name.upper() }}_{{ flag.upper() }}: {{ field.field_type }} = 0x{{ '%.x' | format(loop.index0) }};
    {% endfor %}
    pub const {{ field.name.upper() }}: &'static [&'static str] = &[
    {% for flag in field.flags %}"{{ flag }}", {% endfor %}];
    {% endif %}{% endfor %}

    {# (DE)SERIALIZATION FUNCTIONS #}
    fn deserialize(buf: &[u8]) -> Result<Self, DeserializationError> {
        let mut reader = MessageReader::new(buf)
        {%+ if message.min_len > 0 or message.max_len < 65535 %}.check_len({{ message.min_len }}, {{ message.max_len }}, {{ message.id }}, 0)?{% endif %};

        {% for field in message.fields %}
        {% if field.subfields %}
        let mut {{ field.name }} = Vec::<{{ field.struct_name }}>::with_capacity(reader.remaining() / {{ field.min_len }});
        while reader.remaining() > 0 {
            {% for subfield in field.subfields %}
            let {{ subfield.name }} = reader.{{ read_field(subfield.field_type) }};
            {% endfor %}
            {{ field.name }}.push({{ field.struct_name}} {
                {%+ for subfield in field.subfields %}{{ subfield.name }},{% endfor -%}
            });
        }{% elif field.prefix_type %}
        let {{ field.name }}_len = reader.{{ read_field(field.prefix_type) }} as usize;
        if reader.remaining() < {{ field.name }}_len {
            return Err(DeserializationError{
                user_id: 0,
                message_type: {{ message.id }},
                payload_len: buf.len(),
                error: "{{ message.name }}::{{ field.name }} field is too long",
            });
        }
        let {{ field.name }} = reader.{{ read_field(field.field_type, field.name + "_len") }};
        {% else %}
        let {{ field.name }} = reader.{{ read_field(field.field_type) }};
        {% endif -%}
        {% endfor %}{# field in fields #}

        Ok(Self {
            {% for f in message.fields %}{{ f.name }},{% endfor %}
        })
    }

    fn serialize(&self, user_id: u8) -> Vec<u8> {
        let mut w = MessageWriter::with_expected_payload({{ message.id }}, user_id, {{ payload_len(message) }});
        {% for field in message.fields %}
        {% if field.subfields %}
        for item in self.{{ field.name }}.iter() {
            {% for subfield in field.subfields %}
            w.{{ write_field(subfield.field_type, 'item.' + subfield.name) }};
            {% endfor %}
        }
        {% else %}
        {% if field.prefix_type %}
        w.{{ write_field(field.prefix_type, 'self.' + field.name + '.len() as ' + field.prefix_type) }};
        {% endif %}
        w.{{ write_field(field.field_type, 'self.' + field.name) }};
        {% endif %}{# struct or normal field#}
        {% endfor %}{# field in message.fields #}

        w.into()
    }

    fn to_text(&self, txt: TextMessage) -> TextMessage {
        {% if message.fields[-1].subfields %}{% set dabfield = message.fields[-1] %}
        let mut dabs: Vec<Vec<f64>> = Vec::with_capacity(self.{{ dabfield.name }}.len());
        for dab in self.{{ dabfield.name }}.iter() {
            dabs.push(vec![
                {% for d in dabfield.subfields %}
                dab.{{ d.name }} as f64{% if d.format.startswith('div') %} / {{ d.format[3:] }}.0{% endif %},
                {% endfor %}
            ]);
        }
        {% endif %}{# dabfield #}
        txt
        {% for field in message.fields %}
        {% if field.subfields %}
            .set_dabs(dabs)
        {% else %}
            .{{ textmessage_setfield(field, 'self.' + field.name) }}
        {% endif %}
        {% endfor %}{# field in message.fields #}
    }

    fn from_text(tm: &TextMessage) -> Self {
        {% if message.fields[-1].subfields %}{% set dabfield = message.fields[-1] %}
        let mut dab_structs: Vec<{{ dabfield.struct_name }}> = Vec::with_capacity(tm.dabs.len());
        for dab in tm.dabs.iter() {
            if dab.len() != {{ dabfield.subfields|length }} { continue; }
            dab_structs.push({{ dabfield.struct_name }} {
                {% for d in dabfield.subfields %}
                {{ d.name }}: (dab[{{ loop.index0 }}]{% if d.format.startswith('div') %} * {{ d.format[3:] }}.0{% endif %}) as {{ field_rust_type(d) }},
                {% endfor %}{# dab subfield #}
            });
        }
        {% endif %}{# dabfield #}

        Self{
            {% for field in message.fields %}
            {{ field.name }}: {% if field.subfields %}dab_structs{% else %}{{ textmessage_getfield('tm', field) }}{% endif %},
            {% endfor %}
        }
    }
}

{% endif %}{% endfor %}{# messages with more than one field #}


#[derive(Debug, PartialEq)]
pub enum Body {
    {% for message in messages %}
    {{ comment(message.comment) }}
    {% if message.alias %}
    {{ message.name }}({{ message.alias }}Message),
    {% elif message.fields|length > 1 %}
    {{ message.name }}({{ message.name }}Message),
    {% elif message.fields %}
    {{ message.name }}({{ field_rust_type(message.fields[0]) }}),
    {% else %}
    {{ message.name }},
    {% endif %}

    {% endfor %}{# messages #}
}

#[derive(Debug, PartialEq)]
pub struct Message {
    pub user_id: u8,
    pub body: Body
}

impl Message {
    pub fn deserialize(buf: &[u8]) -> Result<Message, DeserializationError> {
        if buf.len() < 4 {
            return Err(DeserializationError {
                user_id: 0,
                message_type: 0,
                payload_len: 0,
                error: "Message header too short",
            });
        }
        let payload_len = u16::from_be_bytes(buf[0..2].try_into().unwrap()) as usize;
        let message_type = buf[2];
        let user_id = buf[3];

        if buf.len() < 4 + payload_len {
            return Err(DeserializationError {
                user_id,
                message_type,
                payload_len,
                error: "Message truncated",
            });
        }

        let buf = &buf[4..];

        use Body::*;
        Ok(Message {
            user_id,
            body: match message_type {
                {% for message in messages %}
                {{ message.id }} =>
                {% if message.alias %}
                {{ message.name }}({{ message.alias }}Message::deserialize(&buf)?),
                {% elif message.fields|length > 1 %}
                {{ message.name }}({{ message.name }}Message::deserialize(&buf)?),
                {% elif message.fields %}
                {{ message.name }}(MessageReader::new(&buf)
                    {%+ if message.min_len > 0 or message.max_len < 65535 %}.check_len({{ message.min_len }}, {{ message.max_len }}, {{ message.id }}, 0)?{% endif -%}
                    .{{ read_field(message.fields[0].field_type) }}
                ),
                {% else %}
                {{ message.name }},
                {% endif %}
                {% endfor %}{# message in messages #}
                _ => {
                    return Err(DeserializationError {
                        user_id,
                        message_type,
                        payload_len,
                        error: "Unknown message type",
                    });
                }
            }
        })
    }

    pub fn serialize(&self) -> Vec<u8> {
        use Body::*;
        match &self.body {
            {% for message in messages %}
            {% if message.alias or message.fields|length > 1 %}
            {{ message.name }}(b) => b.serialize(self.user_id),
            {% elif message.fields %}
            {{ message.name }}(b) => MessageWriter::single({{ message.id }}, self.user_id, {{ deref_primitive(message.fields[0]) }}b),
            {% else %}
            {{ message.name }} => MessageWriter::with_expected_payload({{ message.id }}, self.user_id, 0).into(),
            {% endif %}
            {% endfor %}{# message in messages #}
        }
    }

    pub fn as_text(&self) -> TextMessage {
        use Body::*;
        match &self.body {
            {% for message in messages %}
            {% if message.alias or message.fields|length > 1%}
            {{ message.name }}(b) => b.to_text(TextMessage::new(self.user_id, "{{ message.cmd_name }}")),
            {% elif message.fields %}
            {{ message.name }}(b) => TextMessage::new(self.user_id, "{{ message.cmd_name }}").{{ textmessage_setfield(message.fields[0], 'b') }},
            {% else %}
            {{ message.name }} => TextMessage::new(self.user_id, "{{ message.cmd_name }}"),
            {% endif %}
            {% endfor %}{# message in messages #}
        }
    }

    pub fn from_text(tm: &TextMessage) -> Option<Self> {
        use Body::*;
        Some(Self{
            user_id: tm.user_id,
            body: match tm.name.as_ref() {
                {% for message in messages %}
                {% if message.alias or message.fields|length > 1%}
                "{{ message.cmd_name }}" => {{ message.name }}({{ message.alias or message.name }}Message::from_text(&tm)),
                {% elif message.fields %}
                "{{ message.cmd_name }}" => {{ message.name }}({{ textmessage_getfield('tm', message.fields[0]) }}),
                {% else %}
                "{{ message.cmd_name }}" => {{ message.name }},
                {% endif %}
                {% endfor %}{# message in messages #}
                _ => { return None; }
            }
        })
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_text().fmt(f)
    }
}

""",
    trim_blocks=True,
    lstrip_blocks=True,
)

def field_rust_type(field):
    if field.field_type == 'Bytes':
        return 'Vec<u8>'
    elif field.field_type == 'struct':
        return f'Vec<{field.struct_name}>'
    elif field.field_type == 'argb32':
        return 'u32'

    return field.field_type

def read_field(ftype, length=None):
    if ftype == 'String':
        if length is None:
            return f"read_remaining_str()"

        return f"read_str({length})"

    elif ftype in ('Vec<u8>', 'Vec<u16>', 'Bytes'):
        if ftype == 'Bytes':
            ftype = 'u8'
        else:
            ftype = ftype[4:2]

        if length is None:
            return f"read_remaining_vec::<{ftype}>()"

        return f"read_vec::<{ftype}>({length})"

    elif ftype == 'argb32':
        ftype = 'u32'

    return f"read::<{ftype}>()"


def write_field(ftype, value):
    if ftype in ('String', 'Vec<u8>', 'Vec<u16>', 'Bytes'):
        value = "&" + value

    return f"write({value})"


def deref_primitive(field):
    if field.field_type in ('u8', 'u16', 'u32', 'i32', 'bool'):
        return '*'
    return ''


def textmessage_setfield(field, name):
    if field.field_type == 'String':
        setter = 'set'
        name = name + ".clone()"
    elif field.field_type == 'Bytes':
        setter = 'set_bytes'
        name = '&' + name
    elif field.field_type == 'Vec<u8>':
        setter = 'set_vec_u8'
        name = '&' + name
    elif field.field_type == 'Vec<u16>':
        setter = 'set_vec_u16'
        name = '&' + name + (", true" if field.format == 'hex' else ", false")
    elif field.field_type == 'argb32':
        setter = 'set_argb32'
    elif hasattr(field, 'flags'):
        return f'set_flags("{field.name}", &Self::{field.name.upper()}, {name})'
    else:
        setter = 'set'
        if field.format.startswith("div"):
            name = f"({name} as f64 / {field.format[3:]}.0).to_string()"
        elif field.format == 'hex':
            digits = int(field.field_type[1:]) // 4 # [ui](8|16|32) -> 2|4|8
            name = f'format!("0x{{:0{digits}x}}", {name})'
        else:
            name = name + ".to_string()"

    return f'{setter}("{field.name}", {name})'


def textmessage_getfield(var, field):
    if field.field_type == 'Bytes':
        getter = 'get_bytes'
    elif field.field_type == 'String':
        return f'{var}.get_str("{field.name}").to_string()'
    elif field.field_type == 'Vec<u8>':
        getter = 'get_vec_u8'
    elif field.field_type == 'Vec<u16>':
        getter = 'get_vec_u16'
    elif field.field_type == 'argb32':
        getter = 'get_argb32'
    elif field.field_type == 'bool':
        return f'{var}.get_str("{field.name}") == "true"'
    elif hasattr(field, 'flags'):
        return f'{var}.get_flags(&Self::{field.name.upper()}, "{field.name}")'
    elif field.field_type in ("u8", "u16", "u32", "i8", "i16", "i32"):
        if field.format.startswith("div"):
            return f'({var}.get_f64("{field.name}") * {field.format[3:]}.0) as {field.field_type}'
        else:
            utype = field.field_type.replace('i', 'u')
            astype = f' as {field.field_type}' if field.field_type.startswith('i') else ''
            return f'{var}.get_{utype}("{field.name}"){astype}'
    else:
        raise ValueError("Unhandled textfield get type: " + field.field_type)

    return f'{var}.{getter}("{field.name}")';


def payload_len(message):
    fixed, vectors = message.length()

    if not vectors:
        return fixed

    total = [str(fixed)]

    for vec in vectors:
        if vec.item_len > 1:
            total.append(f'(self.{vec.name}.len() * {vec.item_len})')
        else:
            total.append(f'self.{vec.name}.len()')

    return '+'.join(total)


def comment(comments):
    return '\n'.join('/// ' + c for c in comments.split('\n'))


if __name__ == '__main__':
    protocol = load_protocol_definition()

    print(template.render(
        messages=protocol['messages'],
        version=protocol['version'],
        field_rust_type=field_rust_type,
        read_field=read_field,
        write_field=write_field,
        deref_primitive=deref_primitive,
        payload_len=payload_len,
        textmessage_setfield=textmessage_setfield,
        textmessage_getfield=textmessage_getfield,
        comment=comment,
    ))
