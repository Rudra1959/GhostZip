/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      fontFamily: {
        sans: ["Aptos", "Segoe UI Variable Text", "Segoe UI", "sans-serif"],
      },
      colors: {
        ink: "#10141a",
        panel: "#171d25",
        panel2: "#202832",
        line: "#2f3a47",
        mint: "#52d6a4",
        amber: "#f4b85a",
        danger: "#ff6b6b",
        sky: "#73b7ff",
      },
    },
  },
  plugins: [],
};
