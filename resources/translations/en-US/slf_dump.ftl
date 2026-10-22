slf-dump-diagnostic-missing-show-option = nothing to show - use --show and/or --history
slf-dump-diagnostic-run-help = run with --help for usage
slf-dump-diagnostic-file-parsing-failure = fail to parse file: {$error}
slf-dump-diagnostic-time-index-out-of-range = time index {$time} out of range (file has {$step_count} steps)
slf-dump-diagnostic-no-such-variable = variable '{$variable}' not found in file
slf-dump-diagnostic-available-variables = available variables:
  { $variables }
slf-dump-diagnostic-no-such-layer = layer #{$requested} does not exist (file has {$actual} {
    $actual ->
    [one] layer
    *[other] layers
})
