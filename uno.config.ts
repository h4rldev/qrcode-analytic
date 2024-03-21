import {
  defineConfig,
  presetAttributify,
  presetIcons,
  presetTypography,
  presetUno,
  presetWind,
  presetWebFonts,
  transformerDirectives,
  presetTagify,
  transformerCompileClass
} from 'unocss'

export default defineConfig({
  shortcuts: [
    // ...
  ],
  theme: {
    colors: {
      // ...
    }
  },
  presets: [
    presetUno(),
    presetAttributify(),
    presetTagify(),
    presetWind(),
    presetIcons({
      autoInstall: true,
    }),
    presetTypography(),
    presetWebFonts({
      fonts: {
        alata: "Alata",
      },
      provider: "bunny",
    }),
  ],
  transformers: [
    transformerDirectives(),
    transformerCompileClass(),
  ],
})
