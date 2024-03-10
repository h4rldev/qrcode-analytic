default: watch

tailwind:
  @rm -fr ./node_modules/qrcode-analytic
  @pnpx tailwindcss -i ./tailwind.scss -o ./styles/styles.css --minify

watch:
  @cargo watch -x "run --release"

build:
  @cargo build --release
  @cp ./target/release/qrcode-analytic .
