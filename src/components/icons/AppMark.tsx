import React from "react";

// Brand mark v4 (mic + halo on warm plate), kept in sync with
// rebrand/generate.mjs. Colors are brand constants, not theme tokens —
// the mark looks the same in both themes, like the app icon.
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
      <rect x="32" y="32" width="960" height="960" rx="212" fill="#F5EFDF" />
      <ellipse
        cx="524"
        cy="238"
        rx="258"
        ry="80"
        fill="none"
        stroke="#2A2419"
        strokeWidth="94"
        transform="rotate(-8 524 238)"
      />
      <ellipse
        cx="524"
        cy="238"
        rx="258"
        ry="80"
        fill="none"
        stroke="#E3A33C"
        strokeWidth="62"
        transform="rotate(-8 524 238)"
      />
      <rect
        x="332"
        y="347"
        width="360"
        height="420"
        rx="180"
        fill="#C4791B"
        stroke="#2A2419"
        strokeWidth="30"
      />
      <path
        d="M 238 587 a 274 274 0 0 0 548 0"
        fill="none"
        stroke="#2A2419"
        strokeWidth="48"
        strokeLinecap="round"
      />
      <line
        x1="512"
        y1="861"
        x2="512"
        y2="895"
        stroke="#2A2419"
        strokeWidth="48"
        strokeLinecap="round"
      />
    </svg>
  );
};

export default AppMark;
