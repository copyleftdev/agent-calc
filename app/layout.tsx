import type { Metadata } from "next";
import {
  Atkinson_Hyperlegible_Next,
  Bricolage_Grotesque,
} from "next/font/google";
import "katex/dist/katex.min.css";
import "./globals.css";

const display = Bricolage_Grotesque({
  subsets: ["latin"],
  weight: "variable",
  axes: ["opsz", "wdth"],
  variable: "--font-display",
});

const body = Atkinson_Hyperlegible_Next({
  subsets: ["latin"],
  weight: "variable",
  style: ["normal", "italic"],
  variable: "--font-body",
});

export const metadata: Metadata = {
  title: "Oddly Exact — Proof over vibes",
  description:
    "Ask a mathematical question. Watch interpretation become deterministic, inspectable evidence.",
  metadataBase: new URL("https://oddly-exact-math.chatgpt.team"),
};

export default function RootLayout({
  children,
}: Readonly<{ children: React.ReactNode }>) {
  return (
    <html lang="en">
      <body className={`${display.variable} ${body.variable}`}>{children}</body>
    </html>
  );
}
