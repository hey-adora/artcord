/** @type {import('tailwindcss').Config} */
module.exports = {
  content: { 
    files: ["*.html", "./src/**/*.rs"],
  },
  theme: {
    extend: {
      colors: {
        "low-purple": "#ECCEFF",
        "mid-purple": "#250157",
        "dark-purple": "#925CB3",
        
      },
    },
  },
  plugins: [],
}
