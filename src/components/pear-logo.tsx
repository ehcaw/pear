import { cn } from "@/lib/utils";

interface PearLogoProps {
  className?: string;
}

export function PearLogo({ className }: PearLogoProps) {
  return (
    <div className={cn("relative", className)}>
      <svg
        viewBox="0 0 100 100"
        fill="none"
        xmlns="http://www.w3.org/2000/svg"
        className="w-full h-full"
      >
        {/* Pear body */}
        <path
          d="M50 95C67.6731 95 82 80.6731 82 63C82 45.3269 67.6731 31 50 31C32.3269 31 18 45.3269 18 63C18 80.6731 32.3269 95 50 95Z"
          fill="url(#pear-gradient)"
        />

        {/* Leaf */}
        <path
          d="M60 25C60 25 70 15 80 15C80 15 75 30 65 35C55 40 50 31 50 31C50 31 55 20 60 25Z"
          fill="url(#leaf-gradient)"
        />

        {/* Stem */}
        <path
          d="M50 31C50 31 52 20 50 15C48 10 45 5 45 5C45 5 42 15 45 25C48 35 50 31 50 31Z"
          fill="#8B5E3C"
        />

        {/* Highlight */}
        <path
          d="M35 55C40 50 50 55 45 65C40 75 30 70 35 55Z"
          fill="white"
          fillOpacity="0.3"
        />

        {/* Gradients */}
        <defs>
          <linearGradient
            id="pear-gradient"
            x1="18"
            y1="63"
            x2="82"
            y2="63"
            gradientUnits="userSpaceOnUse"
          >
            <stop stopColor="#A1F515" />
            <stop offset="1" stopColor="#8AE000" />
          </linearGradient>
          <linearGradient
            id="leaf-gradient"
            x1="50"
            y1="25"
            x2="80"
            y2="15"
            gradientUnits="userSpaceOnUse"
          >
            <stop stopColor="#4D8EFF" />
            <stop offset="1" stopColor="#1A66FF" />
          </linearGradient>
        </defs>
      </svg>
    </div>
  );
}
