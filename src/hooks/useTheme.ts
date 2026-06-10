import { useState, useEffect } from 'react';

export function useTheme() {
  const [theme, setTheme] = useState<"dark" | "light">(() => {
    const saved = localStorage.getItem("ghostzip-theme");
    return (saved as "dark" | "light") || "dark";
  });

  useEffect(() => {
    localStorage.setItem("ghostzip-theme", theme);
    document.documentElement.className = theme;
  }, [theme]);

  return [theme, setTheme] as const;
}
