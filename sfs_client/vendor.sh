mv .cargo .cargol
rm -r vendor
cargo vendor --respect-source-config
mv .cargol .cargo