//const defaultTheme = require('tailwindcss/defaultTheme')

/** @type {import('tailwindcss').Config} */
module.exports = {
  content: {
    files: ["*.html", "./src/**/*.rs"],
  },
  theme: {
    extend: {
      colors: {
        "low-purple": "#ffffff",
        "half-purple": "#67398B",
        "mid-purple": "#925CB3",
        "dark-purple": "#250157",
        "dark2-purple": "#642B87",
        "dark-night": "#1A2625",
        "light-flower": "#E6C5E8"
      },
      backgroundImage: {
        'line-pattern': "url('/assets/bg.svg')",
        'the-star': "url('/assets/star.svg')",
        'sword-lady': "url('/assets/sword_lady.webp')",
      },
      boxShadow: {
        'glowy': '0px 0px 15px 2px #ECCEFF',
      },
      fontFamily: {
        'barcode': ['\"Libre Barcode 128 Text\"'],
      },
    },
  },
  plugins: [],
}
