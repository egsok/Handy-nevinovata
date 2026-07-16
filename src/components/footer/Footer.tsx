import React, { useState, useEffect } from "react";
import { getVersion } from "@tauri-apps/api/app";
import { openUrl } from "@tauri-apps/plugin-opener";
import { Send } from "lucide-react";

import ModelSelector from "../model-selector";
import {
  TELEGRAM_CHANNEL_NAME,
  TELEGRAM_URL_FOOTER,
} from "@/lib/constants/brand";

const Footer: React.FC = () => {
  const [version, setVersion] = useState("");

  useEffect(() => {
    const fetchVersion = async () => {
      try {
        const appVersion = await getVersion();
        setVersion(appVersion);
      } catch (error) {
        console.error("Failed to get app version:", error);
        setVersion("0.1.2");
      }
    };

    fetchVersion();
  }, []);

  return (
    <div className="w-full border-t border-mid-gray/20 pt-3">
      {/* Colophon grid: the press (model) on the left, the publisher (channel)
          standing alone in the center, the run number (version) quiet in the
          corner. Equal 1fr sides keep the link truly centered. */}
      <div className="grid grid-cols-[1fr_auto_1fr] items-center font-mono text-[12px] font-medium px-4 pb-3 text-text/80">
        <div className="flex items-center gap-4 justify-self-start">
          <ModelSelector />
        </div>

        <button
          onClick={() => openUrl(TELEGRAM_URL_FOOTER)}
          className="flex items-center gap-1.5 justify-self-center hover:text-logo-primary transition-colors cursor-pointer"
        >
          <Send size={12} className="shrink-0" />
          <span>{TELEGRAM_CHANNEL_NAME}</span>
        </button>

        {/* eslint-disable-next-line i18next/no-literal-string */}
        <span className="justify-self-end text-text/55">v{version}</span>
      </div>
    </div>
  );
};

export default Footer;
