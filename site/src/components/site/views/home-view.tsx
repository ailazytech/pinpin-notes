"use client";

import { Hero } from "../sections/hero";
import { StickerMarquee } from "../sections/marquee";
import { VideoSection } from "../sections/video-section";
import { FeaturesOverview } from "../sections/features-overview";
import { Chapters } from "../sections/chapters";
import { DetailsGrid } from "../sections/details-grid";
import { PowerBand } from "../sections/power-band";
import { LanguagesSection } from "../sections/languages-section";
import { DownloadSection } from "../sections/download-section";
import type { NavigateFn } from "../header";

export function HomeView({
  navigate,
  onDownload,
}: {
  navigate: NavigateFn;
  onDownload: () => void;
}) {
  return (
    <>
      <Hero navigate={navigate} onDownload={onDownload} />
      <StickerMarquee />
      <VideoSection />
      <FeaturesOverview navigate={navigate} />
      <Chapters />
      <DetailsGrid navigate={navigate} />
      <PowerBand />
      <LanguagesSection />
      <DownloadSection onDownload={onDownload} />
    </>
  );
}
