.PHONY:readme
readme:
	@awk -i inplace -v q="\`\`\`" 'BEGIN {p=1} /^<!-- help start -->/{print;print "";print q;print "$$ shiv --help";system("cargo run -- --help");print q;print "";p=0} /^<!-- help end -->/{p=1} p' README.md

