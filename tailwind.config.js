/** @type {import('tailwindcss').Config} */
module.exports = {
  content: { 
    files: ["*.html", "./src/**/*.rs"],
  },
  theme: {
    extend: {
      colors: {
        "low-purple": "#ECCEFF",
        "half-purple": "#67398B",
        "mid-purple": "#925CB3",
        "dark-purple": "#250157",  
        "dark2-purple": "#642B87",
      },
      backgroundImage: {
        'line-pattern': "url('/assets/bg.svg')",
      }
    },
  },
  plugins: [],
}
