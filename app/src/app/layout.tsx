import type { Metadata } from "next";
import "./styles.css";

export const metadata: Metadata = {
  title: "RWA Registry · Fictional Demo",
  description: "A local demonstration of composable property, placement, and distribution registries.",
};

export default function RootLayout({ children }: Readonly<{ children: React.ReactNode }>) {
  return <html lang="en"><body>{children}</body></html>;
}
