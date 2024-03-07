default: watch

tailwind:
  @rm -fr ./node_modules/qrcode-analytic
  @pnpx tailwindcss -i ./tailwind.scss -o ./styles/styles.css --build

watch:
  @cargo watch -x run
