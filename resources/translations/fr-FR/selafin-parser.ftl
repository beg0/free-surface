### Fichier de traduction pour storage::selafin::parser

## Nom d’un enregistrement du fichier Selafin

selafin-record-var-cnt = nombre de variables
selafin-record-var = variable
selafin-record-title = titre
selafin-record-geometry-header = en-tête de géometrie
selafin-record-iparam = IPARAMS
selafin-record-datetime = date et heure
selafin-record-ikle3 = table de connectivité
selafin-record-ipob3 = table des frontières

## Diagnostics (voir l’énumération SelafinParserDiag)
selafin-parser-diag-unknown-endianness-error = Impossible de déterminer l’ordre des octets du fichier à partir du premier enregistrement
selafin-parser-diag-record-len-mismatch = Longueur d’enregistrement incohérente : en-tête={$len}, pied={$ctrl}
selafin-parser-diag-float-size-error = Impossible de déterminer la taille des nombres flottants : l’enregistrement fait {$record_size} octets, alors que la taille attendue est {$expected_f32} (f32) ou {$expected_f64} (f64) pour npoin3={$npoin3}

selafin-parser-diag-invalid-date = Date invalide dans l’enregistrement datetime : { NUMBER($year, minimumIntegerDigits: 4) }-{ NUMBER($month, minimumIntegerDigits: 2) }-{ NUMBER($day, minimumIntegerDigits: 2) }
selafin-parser-diag-invalid-time = Heure invalide dans l’enregistrement datetime : { NUMBER($hour, minimumIntegerDigits: 2) }:{ NUMBER($min, minimumIntegerDigits: 2) }:{ NUMBER($sec, minimumIntegerDigits: 2) }
selafin-parser-diag-inconsistent-coords-vector = Nombre de points incohérent. L’en-tête indique {$npoin3} points, l’enregistrement x en indique {$x_len} et l’enregistrement y en indique {$y_len}
selafin-parser-diag-unexpected-entry-cnt = L’enregistrement «{$record_name}» contient {$got} valeurs, alors que {$expected} sont attendues
selafin-parser-diag-truncated-record = L’enregistrement «{$record_name}» est tronqué. Nombre obtenu : { WITH_UNITS($got, $unit) } ; au moins { WITH_UNITS($expected, $unit) } étaient attendus
selafin-parser-diag-inconsistent-plan-cnt = Nombre de plans incohérent : IPARAMS en indique {$iparam_nplan}, la géométrie en indique {$geo_nplan}.
selafin-parser-diag-empty-time-entry = L’entrée temporelle de l’historique n°{$time_idx} est vide
selafin-parser-diag-empty-history-record = Les enregistrements de valeurs de la variable «{$var_name}» pour l’entrée d’historique n°{$time_idx} contiennent {$var_cnt} valeurs, alors que {$npoin3} sont attendues
