#!/usr/bin/env python3

from yaml import safe_load as load_yaml

import sys


class BadDefinition(Exception):
    pass


class Message:
    def __init__(self, name, desc, messages):
        self.name = name
        self.cmd_name = desc.get('name', name.lower())
        self.id = _required(name, desc, 'id', int)
        self.comment = str(desc.get('comment', ''))

        if 'alias' in desc:
            try:
                self.alias = desc['alias']
            except KeyError:
                raise BadDefinition(f"Message {alias} not found!")
            self.fields = []

        else:
            fields = _required(name, desc, 'fields', list)

            self.alias = None
            self.fields = []
            self.min_len = 0
            self.max_len = 0
            last = len(fields) - 1
            names = set()

            for idx, field in enumerate(fields):
                new_field = make_field(self, field, idx == last, self.min_len, self.max_len)
                if new_field.name in names:
                    raise BadDefinition(f"Duplicate field name '{new_field.name}' in {self.name}")
                names.add(new_field.name)

                self.fields.append(new_field)
                self.min_len += new_field.min_len
                self.max_len = min(0xffff, self.max_len + new_field.max_len)

    @property
    def is_fixed_len(self):
        return self.min_len == self.max_len

    def length(self):
        """The length consists of two parts:

        - the sum of all fixed fields
        - vector fields (length is vector len * field.item_len)
        """

        fixed = 0
        arrays = []
        for field in self.fields:
            if field.is_fixed_len:
                fixed += field.min_len
            else:
                arrays.append(field)

        return (fixed, arrays)


    def __repr__(self):
        out = []
        if self.comment:
            out += list('# ' + line for line in self.comment.split('\n'))

        if self.alias:
            out.append(f'{self.name}(name="{self.cmd_name}") --> {self.alias}')

        elif self.fields:
            if self.is_fixed_len:
                out.append(f'{self.name}(name="{self.cmd_name}", id={self.id}, len={self.min_len}):')
            else:
                out.append(f'{self.name}(name="{self.cmd_name}", id={self.id}, minlen={self.min_len}, maxlen={self.max_len}):')
            out += list('\t' + repr(f) for f in self.fields)

        else:
            out.append(f'{self.name}(name="{self.cmd_name}", id={self.id})')
        return '\n'.join(out)


def make_field(message, desc, is_last, prev_minlen, prev_maxlen):
    if isinstance(desc, str):
        try:
            fieldname, typename = desc.split()
        except ValueError:
            raise BadDefinition(f"Invalid short form field '{desc}'")
        desc = {}

    elif isinstance(desc, dict):
        for key, value in desc.items():
            if ' ' in key:
                try:
                    fieldname, typename = key.split()
                except ValueError:
                    raise BadDefinition(f"Invalid long form field '{key}'")
                desc['_type_params'] = value
                break
        else:
            raise BadDefinition("Field name not found in long form field")

    else:
        raise BadDefinition("Field should be a string or an object")

    try:
        fieldcls = globals()[f'Field{typename.capitalize()}']
    except KeyError:
        raise BadDefinition(f"Type class for type '{typename}' not found!")

    return fieldcls(
        fieldname,
        attributes=desc,
        is_last=is_last,
        prev_minlen=prev_minlen,
        prev_maxlen=prev_maxlen,
    )


class Field:
    def __init__(self, name):
        self.name = name
        self.format = ''

    @property
    def is_fixed_len(self):
        return self.min_len == self.max_len

    def __repr__(self):
        if self.is_fixed_len:
            return f'[{self.min_len}] {self.name}: {self.field_type}'

        return f'[{self.min_len}-{self.max_len}] {self.name}: {self.field_type}'


class IntegerField(Field):
    def __init__(self, name, attributes, **kwargs):
        super().__init__(name)
        self.format = attributes.get('format', attributes.get('_type_params', ''))

    @classmethod
    def F(cls, field_type, size):
        return type(f'Field{field_type.capitalize()}', (cls,), {
            'min_len': size,
            'max_len': size,
            'field_type': field_type}
        )


FieldI8 = IntegerField.F('i8', 1)
FieldI16 = IntegerField.F('i16', 2)
FieldI32 = IntegerField.F('i32', 4)
FieldU8 = IntegerField.F('u8', 1)
FieldU16 = IntegerField.F('u16', 2)
FieldU32 = IntegerField.F('u32', 4)
FieldArgb32 = IntegerField.F('argb32', 4)

FieldBool = IntegerField.F('bool', 1)


class FieldFlags(Field):
    def __init__(self, name, attributes, **kwargs):
        super().__init__(name)
        self.flags = [(v, 1<<i) for i,v in enumerate(attributes['_type_params'])]
        if len(self.flags) <= 8:
            minlen = 1
        elif len(self.flags) <= 16:
            minlen = 2
        elif len(self.flags) <= 32:
            minlen = 4
        else:
            raise BadDefinition("Too many flags")

        self.min_len = int(attributes.get('length', minlen))
        self.max_len = self.min_len
        if self.min_len < minlen:
            raise BadDefinition("Length is too short")

    @property
    def field_type(self):
        if self.min_len == 1:
            return 'u8'
        elif self.min_len == 2:
            return 'u16'
        elif self.min_len == 4:
            return 'u32'
        else:
            raise BadDefinition(f"Unsupported flags field size: {self.size}")

    def __repr__(self):
        return super().__repr__() + ' ' + str(self.flags)


class FieldBytes(Field):
    field_type = 'Bytes'
    item_len = 1

    def __init__(self, name, attributes, is_last, prev_minlen, prev_maxlen, **kwargs):
        super().__init__(name)

        self.format = attributes.get('format', attributes.get('_type_params', ''))

        if is_last:
            self.prefix_type = None
            self.min_len = int(attributes.get('min_len', 0))
            self.max_len = min(0xffff - prev_minlen, int(attributes.get('max_len', 0xffff)))

        else:
            self.prefix_type = attributes.get('prefix_type', 'u8')
            if self.prefix_type not in ('u8', 'u16'):
                raise BadDefinition("Prefix type can only be u8 or u16")
            prefix_len = int(self.prefix_type[1:]) // 8
            self.min_len = prefix_len + int(attributes.get('min_len', 0))
            self.max_len = min(
                (1<<prefix_len*8) - 1 + prefix_len,
                0xffff-prev_minlen,
                int(attributes.get('max_len', 0xffff))+1
            )
            if self.max_len < self.min_len:
                raise BadDefinition(f"{self.name}: Max length smaller than min length!")

    def __repr__(self):
        len_range = f'[{self.min_len}-{self.max_len}]'
        if self.prefix_type:
            return f'{len_range} {self.name}: {self.prefix_type} + {self.field_type}'
        else:
            return f'{len_range} {self.name}: {self.field_type}'


# Note: This is typically a synonym for FieldBytes
class FieldVec_u8(FieldBytes):
    field_type = 'Vec<u8>'


class FieldVec_u16(FieldBytes):
    field_type = 'Vec<u16>'
    item_len = 2


class FieldUtf8(FieldBytes):
    field_type = 'String'


class FieldStruct(Field):
    field_type = 'struct'

    def __init__(self, name, attributes, is_last, prev_minlen, prev_maxlen):
        super().__init__(name)

        if not is_last:
            raise BadDefinition("struct field must be the last field")

        if prev_minlen != prev_maxlen:
            raise BadDefinition("struct field should be the only variable length field in a message")

        self.struct_name = _required(name, attributes, 'name', str)
        self.subfields = []
        subfields = _required(name, attributes, 'fields', list)

        last = len(subfields)
        length = 0
        names = set()

        for idx, field in enumerate(subfields):
            subfield = make_field(self, field, idx == last, length, length)
            if subfield.name in names:
                raise BadDefinition(f"Duplicate subfield name '{subfield.name}' in {name}")
            if not subfield.is_fixed_len:
                raise BadDefinition(f"{name}.{subfield.name}: only fixed length fields allowed in structs")
            names.add(subfield.name)

            self.subfields.append(subfield)
            length += int(subfield.min_len)

        self.item_len = length
        self.min_len = length
        self.max_len = (0xffff - prev_minlen) // length * length
        self.max_items = self.max_len // length

    def __repr__(self):
        out = [f"{super().__repr__()} {self.struct_name} * 1..{self.max_items}"]
        out += list('\t\t' + repr(f) for f in self.subfields)
        return '\n'.join(out)


def _required(name, obj, key, valuetype):
    try:
        val = obj[key]
    except KeyError:
        raise BadDefinition(f"{name} is missing required attribute {key}")

    if not isinstance(val, valuetype):
        raise BadDefinition(f"{name} property {key} must be of type {valuetype}")

    return val


def load_protocol_definition():
    with open('protocol.yaml') as f:
        desc = load_yaml(f)

    protocol = {}
    messages = {}
    ids = set()
    cmd_names = set()
    for msg in desc:
        if msg == '_protocol':
            protocol = desc[msg]
            continue

        m = Message(msg, desc[msg], messages)
        if m.name in messages:
            raise BadDefinition(f"{m.name}: duplicate message name!")
        if m.cmd_name in cmd_names:
            raise BadDefinition(f"{m.name}: duplicate message command name {m.cmd_name}!")
        if m.id in ids:
            raise BadDefinition(f"{m.name}: duplicate message ID {m.id}!")

        messages[m.name] = m
        ids.add(m.id)
        cmd_names.add(m.cmd_name)

    return {
        **protocol,
        'messages': sorted(messages.values(), key=lambda x: x.id)
    }


if __name__ == '__main__':
    proto = load_protocol_definition()
    print("Protocol version:", proto['version'])
    for m in proto['messages']:
        print(repr(m))
        print("")

