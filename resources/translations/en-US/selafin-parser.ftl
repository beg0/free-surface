### Translation file for storage::selafin::parser

## Name of some record in selafin file

selafin-record-var-cnt = variable count
selafin-record-var = variable
selafin-record-title = title
selafin-record-geometry-header = geometry header
selafin-record-iparam = IPARAMS
selafin-record-datetime = date and time
selafin-record-ikle3 = connectivity table
selafin-record-ipob3 = boundary table

## Diagnostics (see enum SelafinParserDiag)
selafin-parser-diag-unknown-endianness-error = Cannot determine file endianness from the first record
selafin-parser-diag-record-len-mismatch = Record length mismatch: header={$len} tailer={$ctrl}
selafin-parser-diag-float-size-error = Cannot determine float size: record is {$record_size} bytes, expected {$expected_f32} (f32) or {$expected_f64} (f64) for npoin3={$npoin3}

selafin-parser-diag-invalid-date = Invalid date in datetime record: { NUMBER($year, minimumIntegerDigits: 4) }-{ NUMBER($month, minimumIntegerDigits: 2) }-{ { NUMBER($day, minimumIntegerDigits: 2) }}
selafin-parser-diag-invalid-time = Invalid time in datetime record: { NUMBER($hour, minimumIntegerDigits: 2) }:{ NUMBER($min, minimumIntegerDigits: 2) }:{ { NUMBER($sec, minimumIntegerDigits: 2) }}
selafin-parser-diag-inconsistent-coords-vector = Inconsistent number of points. Header says {$npoin3} points, x record says {$x_len} points, y records says {$y_len}
selafin-parser-diag-unexpected-entry-cnt = "{$record_name}" record has {$got} values, expected {$expected}
selafin-parser-diag-truncated-record = "{$record_name}" record truncated. Got { WITH_UNITS($got, $unit) } and expected at least { WITH_UNITS($expected, $unit)}
selafin-parser-diag-inconsistent-plan-cnt = Inconsistent number of planes, IPARAMS says {$iparam_nplan}, geometry says {$geo_nplan}.
selafin-parser-diag-empty-time-entry = Time entry for history #{$time_idx} is empty
selafin-parser-diag-empty-history-record = values of Variable '{$var_name}' value records for history entry #{$time_idx} has {$var_cnt} values, expected {$npoin3}
