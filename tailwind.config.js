const { addDynamicIconSelectors } = require('@iconify/tailwind');

/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./*.html", "./**/*.html", "*.css", "./**/*.css" 
  ],
  theme: {
    extend: {
      FontFace: {
        "Ubuntu Nerd Font Propo": "url('/fonts/UbuntuNerdFontPropo-Regular.ttf')",
        "FiraCode Nerd Font Mono": "url('/fonts/FiraCodeNerdFontMono-Regular.ttf')",
      },
      fontFamily: {
        "firacode": '"FiraCode Nerd Font Mono", monospace',
        "ubuntu": '"Ubuntu Nerd Font Propo", sans-serif',
      },
    },
  },
  plugins: [
    addDynamicIconSelectors(),
  ],
}

