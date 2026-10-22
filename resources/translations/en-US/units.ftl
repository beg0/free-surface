### Translation file to feed the built-in functions WITH_UNITS() and UNITS()
###
### Function WITH_UNITS($cnt, $unit) will call message `with-unit-$unit` with arument `$cnt`
### Message `with-unit-$unit` is used to print the value `$cnt` with its unit `$unit`

### Function UNITS($cnt, $unit) will call message `unit-$unit` with arument `$cnt`
### Message `unit-$unit` is used to print only the unit (without value). The `$cnt` is only
### provided for plurial rule

## Support of WITH_UNITS(). There is a single argument: `$cnt`
with-unit-byte = { $cnt } { UNITS($cnt, "byte") }

with-unit-u32 = { $cnt } { UNITS($cnt, "u32") }

## Support of UNITS(). There is a single argument: `$cnt`

unit-byte = { $cnt ->
    [one] byte
   *[other] bytes
}

unit-u32 = { $cnt ->
    [one] word of 32-bits
   *[other] words of 32-bits
}
