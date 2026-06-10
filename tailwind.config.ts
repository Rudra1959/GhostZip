import type { Config } from "tailwindcss";

export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      fontFamily: {
        sans: ["Inter", "Segoe UI", "system-ui", "sans-serif"]
      },
      colors: {
        ink: "#10141a",
        panel: "#171d25",
        panel2: "#202832",
        line: "#2f3a47",
        mint: "#52d6a4",
        amber: "#f4b85a",
        danger: "#ff6b6b",
        sky: "#73b7ff"
      }
    }
  },
  plugins: []
} satisfies Config;
