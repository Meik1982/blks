complete -c blks -s c -l check -d 'Read checksums from the specified file and check them' -r -F
complete -c blks -l format -d 'Digest output format' -r -f -a "base64\t'64-character standard Base64 (matches SHA-256 hex line width, zero padding)'
base64-url\t'64-character URL-safe Base64 (using - and _ instead of + and /)'
hex\t'96-character hexadecimal digest'
raw\t'Raw 48-byte binary digest to stdout'"
complete -c blks -s j -l threads -d 'Number of worker threads for parallel tree reduction (default: all logical cores)' -r
complete -c blks -l completion -d 'Generate shell auto-completion script (bash, zsh, fish, powershell, elvish)' -r -f -a "bash\t''
elvish\t''
fish\t''
powershell\t''
zsh\t''"
complete -c blks -l hex -d 'Convenience flag for hexadecimal output (equivalent to --format hex)'
complete -c blks -l raw -d 'Convenience flag for raw binary output to stdout'
complete -c blks -s q -l quiet -d 'Quiet mode: in check mode, only print failed files'
complete -c blks -l json -d 'Output machine-readable JSON telemetry'
complete -c blks -l benchmark -d 'Benchmark mode: print elapsed time and throughput in GB/s'
complete -c blks -l no-mmap -d 'Disable memory-mapping (forces streaming reads; prevents SIGBUS on network shares or volatile files)'
complete -c blks -s h -l help -d 'Print help (see more with \'--help\')'
complete -c blks -s V -l version -d 'Print version'
