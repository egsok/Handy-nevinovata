import React, { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { commands } from "@/bindings";
import { SettingContainer } from "../../ui/SettingContainer";

interface DebugPathsProps {
  descriptionMode?: "tooltip" | "inline";
  grouped?: boolean;
}

export const DebugPaths: React.FC<DebugPathsProps> = ({
  descriptionMode = "inline",
  grouped = false,
}) => {
  const { t } = useTranslation();
  const [appDir, setAppDir] = useState<string>("");

  useEffect(() => {
    const loadAppDir = async () => {
      try {
        const result = await commands.getAppDirPath();
        if (result.status === "ok") {
          setAppDir(result.data);
        }
      } catch (err) {
        console.error("Failed to load app directory:", err);
      }
    };

    loadAppDir();
  }, []);

  return (
    <SettingContainer
      title="Debug Paths"
      description="Display internal file paths and directories for debugging purposes"
      descriptionMode={descriptionMode}
      grouped={grouped}
    >
      <div className="text-sm text-gray-600 space-y-2">
        <div>
          <span className="font-medium">
            {t("settings.debug.paths.appData")}
          </span>{" "}
          <span className="font-mono text-xs select-text">{appDir}</span>
        </div>
        <div>
          <span className="font-medium">
            {t("settings.debug.paths.models")}
          </span>{" "}
          <span className="font-mono text-xs select-text">
            {appDir && `${appDir}/models`}
          </span>
        </div>
        <div>
          <span className="font-medium">
            {t("settings.debug.paths.settings")}
          </span>{" "}
          <span className="font-mono text-xs select-text">
            {appDir && `${appDir}/settings_store.json`}
          </span>
        </div>
      </div>
    </SettingContainer>
  );
};
