default: watch

tailwind:
  @pnpx tailwindcss -i ./tailwind.scss -o ./styles/styles.css --watch

watch:
  @cargo watch -x run
