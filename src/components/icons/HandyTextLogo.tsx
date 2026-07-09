import React from "react";

const HandyTextLogo = ({
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
      viewBox="0 0 940 140"
      xmlns="http://www.w3.org/2000/svg"
    >
      <text
        x="0"
        y="102"
        fontFamily="Consolas, 'Cascadia Mono', ui-monospace, monospace"
        fontSize="96"
      >
        {/* eslint-disable-next-line i18next/no-literal-string */}
        <tspan fontWeight="700" className="logo-primary">klava</tspan>
        {/* eslint-disable-next-line i18next/no-literal-string */}
        <tspan fill="var(--color-mid-gray)">-nevinovata</tspan>
      </text>
      <rect
        x="858"
        y="26"
        width="24"
        height="88"
        fill="var(--color-logo-primary)"
      />
    </svg>
  );
};

export default HandyTextLogo;
