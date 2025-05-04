import type { Config } from "tailwindcss";
const config: Config = {
  darkMode: "class",
  content: [
    "./pages/**/*.{ts,tsx}",
    "./components/**/*.{ts,tsx}",
    "./app/**/*.{ts,tsx}",
    "./src/**/*.{ts,tsx}",
    "*.{js,ts,jsx,tsx,mdx}",
  ],
  theme: {
    extend: {
      colors: {
        border: "hsl(var(--border))",
        input: "hsl(var(--input))",
        ring: "hsl(var(--ring))",
        background: "hsl(var(--background))",
        foreground: "hsl(var(--foreground))",
        primary: {
          DEFAULT: "hsl(var(--primary))",
          foreground: "hsl(var(--primary-foreground))",
        },
        secondary: {
          DEFAULT: "hsl(var(--secondary))",
          foreground: "hsl(var(--secondary-foreground))",
        },
        destructive: {
          DEFAULT: "hsl(var(--destructive))",
          foreground: "hsl(var(--destructive-foreground))",
        },
        muted: {
          DEFAULT: "hsl(var(--muted))",
          foreground: "hsl(var(--muted-foreground))",
        },
        accent: {
          DEFAULT: "hsl(var(--accent))",
          foreground: "hsl(var(--accent-foreground))",
        },
        popover: {
          DEFAULT: "hsl(var(--popover))",
          foreground: "hsl(var(--popover-foreground))",
        },
        card: {
          DEFAULT: "hsl(var(--card))",
          foreground: "hsl(var(--card-foreground))",
        },
        cream: {
          50: "#FFFDF7",
          100: "#FFF9E8",
          200: "#FFF3D1",
          300: "#FFEAB7",
          400: "#FFE29D",
          500: "#FFD97F",
        },
        pear: {
          50: "#F4FFE8",
          100: "#E9FFC2",
          200: "#D3FF8C",
          300: "#B8FF47",
          400: "#A1F515",
          500: "#8AE000",
        },
        zed: {
          50: "#EEF5FF",
          100: "#D9EAFF",
          200: "#B3D5FF",
          300: "#80B5FF",
          400: "#4D8EFF",
          500: "#1A66FF",
          600: "#0047CC",
          700: "#003399",
          800: "#002266",
          900: "#001133",
        },
      },
      borderRadius: {
        lg: "var(--radius)",
        md: "calc(var(--radius) - 2px)",
        sm: "calc(var(--radius) - 4px)",
      },
      backgroundImage: {
        "gradient-radial": "radial-gradient(var(--tw-gradient-stops))",
        noise: "url('/noise.png')",
      },
      animation: {
        "bounce-slow": "bounce 3s infinite",
        "pulse-slow": "pulse 4s cubic-bezier(0.4, 0, 0.6, 1) infinite",
        float: "float 6s ease-in-out infinite",
        wave: "wave 2.5s infinite",
        shimmer: "shimmer 2s linear infinite",
      },
      keyframes: {
        float: {
          "0%, 100%": { transform: "translateY(0)" },
          "50%": { transform: "translateY(-10px)" },
        },
        wave: {
          "0%": { transform: "rotate(0deg)" },
          "20%": { transform: "rotate(14deg)" },
          "40%": { transform: "rotate(-8deg)" },
          "60%": { transform: "rotate(14deg)" },
          "80%": { transform: "rotate(-4deg)" },
          "100%": { transform: "rotate(0deg)" },
        },
        shimmer: {
          "0%": { backgroundPosition: "-200% 0" },
          "100%": { backgroundPosition: "200% 0" },
        },
      },
    },
  },
  plugins: [require("tailwindcss-animate")],
};

export default config;
