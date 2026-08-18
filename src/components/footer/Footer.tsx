import React, { useState, useEffect } from "react";
import { getVersion } from "@tauri-apps/api/app";
import { openUrl } from "@tauri-apps/plugin-opener";
import { Send } from "lucide-react";

import ModelSelector from "../model-selector";
import UpdateChecker from "../update-checker";
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
          corner. Equal 1fr sides keep the link truly centered; the column gap
          keeps a long update status from gluing itself to the channel link
          (the status text truncates, the link never moves). */}
      <div className="grid grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)] gap-x-4 items-center font-mono text-[12px] font-medium px-4 pb-3 text-text/80">
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

        <div className="flex items-center gap-1 justify-self-end min-w-0 text-text/55">
          <UpdateChecker />
          {/* eslint-disable-next-line i18next/no-literal-string */}
          <span className="shrink-0">v{version}</span>
        </div>
      </div>
    </div>
  );
};

export default Footer;
