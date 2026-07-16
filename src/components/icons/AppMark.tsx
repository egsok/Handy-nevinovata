import React from "react";

// Brand mark v5 (mic + halo, two-ink print), kept in sync with
// rebrand/generate.mjs. Colors are brand constants, not theme tokens —
// the mark looks the same in both themes, like the app icon.
// The magenta halo is printed +24 units off the violet outline (1.22px at the
// mark's one live size, 52px): the inks didn't register — не виновата.
const AppMark = ({
  width,
  height,
  className,
}: {
  width?: number;
  height?: number;
  className?: string;
}) => {
  return (
    <svg
      width={width}
      height={height}
      className={className}
      viewBox="0 0 1024 1024"
      xmlns="http://www.w3.org/2000/svg"
    >
      <rect x="32" y="32" width="960" height="960" rx="212" fill="#f2ecdc" />
      <ellipse
        cx="524"
        cy="238"
        rx="258"
        ry="80"
        fill="none"
        stroke="#2c1a72"
        strokeWidth="94"
        transform="rotate(-8 524 238)"
      />
      <ellipse
        cx="548"
        cy="262"
        rx="258"
        ry="80"
        fill="none"
        stroke="#e11b76"
        strokeWidth="62"
        transform="rotate(-8 548 262)"
      />
      <rect x="332" y="347" width="360" height="420" rx="180" fill="#2c1a72" />
      <path
        d="M 238 587 a 274 274 0 0 0 548 0"
        fill="none"
        stroke="#2c1a72"
        strokeWidth="48"
        strokeLinecap="round"
      />
      <line
        x1="512"
        y1="861"
        x2="512"
        y2="895"
        stroke="#2c1a72"
        strokeWidth="48"
        strokeLinecap="round"
      />
    </svg>
  );
};

export default AppMark;
