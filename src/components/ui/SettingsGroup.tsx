import React from "react";

interface SettingsGroupProps {
  title?: string;
  description?: string;
  children: React.ReactNode;
}

export const SettingsGroup: React.FC<SettingsGroupProps> = ({
  title,
  description,
  children,
}) => {
  return (
    <div className="space-y-2">
      {title && (
        <div className="px-4">
          <h2 className="font-mono text-[11px] font-medium text-mid-gray uppercase tracking-[0.14em]">
            {title}
          </h2>
          {description && (
            <p className="text-xs text-mid-gray mt-1">{description}</p>
          )}
        </div>
      )}
      {/* No border: the sheet is already a shade off the ground, and outlining it
          too cut the page into slabs. Colour alone marks the edge. */}
      <div className="bg-surface rounded-lg overflow-visible">
        <div className="divide-y divide-mid-gray/20">{children}</div>
      </div>
    </div>
  );
};
