//const defaultTheme = require('tailwindcss/defaultTheme')

/** @type {import('tailwindcss').Config} */
module.exports = {
  content: {
    files: ["*.html", "./artcord-leptos/**/*.rs"],
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
        "dark-night2": "#1A2621",
        "light-flower": "#E6C5E8",
        "second-one": "#823679",
        "first-one": "#41355E",
      },
      backgroundImage: {
        "line-pattern": "url('/assets/bg.svg')",
        "the-star": "url('/assets/star.svg')",
        "sword-lady": "url('/assets/sword_lady.webp')",
        "sword-ico": "url('/assets/sword.svg')",
      },
      boxShadow: {
        glowy: "0px 0px 15px 2px #ECCEFF",
      },
      fontFamily: {
        barcode: ['"Libre Barcode 128 Text"'],
      },
      screens: {
        desktop: "1700px",
      },
    },
  },
  plugins: [],
};
