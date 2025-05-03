export function WaveBackground() {
  return (
    <div className="absolute inset-0 z-0 overflow-hidden pointer-events-none">
      <div className="absolute inset-0 bg-noise opacity-[0.015] z-10"></div>

      {/* Top wave */}
      <div className="absolute top-0 left-0 right-0 h-64 overflow-hidden">
        <svg
          viewBox="0 0 1200 120"
          preserveAspectRatio="none"
          className="absolute top-0 left-0 w-[200%] h-64 text-zed-50 dark:text-zed-900/30 rotate-180"
          style={{ transform: "rotate(180deg) translateY(-50%)" }}
        >
          <path
            d="M321.39,56.44c58-10.79,114.16-30.13,172-41.86,82.39-16.72,168.19-17.73,250.45-.39C823.78,31,906.67,72,985.66,92.83c70.05,18.48,146.53,26.09,214.34,3V0H0V27.35A600.21,600.21,0,0,0,321.39,56.44Z"
            className="fill-current"
          ></path>
        </svg>
      </div>

      {/* Bottom wave */}
      <div className="absolute bottom-0 left-0 right-0 h-64 overflow-hidden">
        <svg
          viewBox="0 0 1200 120"
          preserveAspectRatio="none"
          className="absolute bottom-0 left-0 w-[200%] h-64 text-cream-100 dark:text-zed-900/20"
        >
          <path
            d="M321.39,56.44c58-10.79,114.16-30.13,172-41.86,82.39-16.72,168.19-17.73,250.45-.39C823.78,31,906.67,72,985.66,92.83c70.05,18.48,146.53,26.09,214.34,3V120H0V92.65A600.21,600.21,0,0,0,321.39,56.44Z"
            className="fill-current"
          ></path>
        </svg>
      </div>

      {/* Floating circles */}
      <div className="absolute top-20 left-[10%] w-32 h-32 rounded-full bg-zed-100 dark:bg-zed-800/20 blur-3xl opacity-70 animate-float"></div>
      <div
        className="absolute top-40 right-[15%] w-40 h-40 rounded-full bg-cream-200 dark:bg-zed-900/30 blur-3xl opacity-50 animate-float"
        style={{ animationDelay: "-2s" }}
      ></div>
      <div
        className="absolute bottom-20 left-[20%] w-36 h-36 rounded-full bg-zed-50 dark:bg-zed-800/10 blur-3xl opacity-60 animate-float"
        style={{ animationDelay: "-4s" }}
      ></div>
    </div>
  );
}
