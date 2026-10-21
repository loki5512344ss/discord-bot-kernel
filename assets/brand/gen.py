#!/usr/bin/env python3
"""Generates the bot's banners and avatar (SVG -> PNG via rsvg-convert). Run from this directory."""
import subprocess

MARK = '<path d="M24 4l18 20-18 20L6 24z" fill="url(#face)" mask="url(#cut)"/>'
DEFS = """
<defs>
  <linearGradient id="face" x1="0" y1="0" x2="0.4" y2="1"><stop offset="0" stop-color="#ffb27a"/><stop offset="1" stop-color="#ff5d1f"/></linearGradient>
  <mask id="cut"><rect width="48" height="48" fill="#fff"/><path d="M24 2V46" stroke="#000" stroke-width="3"/>
    <path d="M16 22l8 4 8-4" stroke="#000" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"/></mask>
  <radialGradient id="fade" cx="82%" cy="50%" r="55%"><stop offset="0" stop-color="#fff"/><stop offset="0.6" stop-color="#fff" stop-opacity="0.35"/><stop offset="1" stop-color="#000"/></radialGradient>
  <mask id="halo"><rect width="W" height="H" fill="url(#fade)"/></mask>
  <pattern id="dots" width="8" height="8" patternUnits="userSpaceOnUse"><rect x="1" y="1" width="4" height="4" fill="#ff6a2b"/><rect x="5" y="5" width="2" height="2" fill="#ff6a2b"/></pattern>
</defs>"""

def banner(name, title, sub, tag):
    w, h = 1000, 280
    svg = f"""<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}">
{DEFS.replace('"W"', f'"{w}"').replace('"H"', f'"{h}"')}
<rect width="{w}" height="{h}" fill="#100f0e"/>
<rect width="{w}" height="{h}" fill="url(#dots)" mask="url(#halo)" opacity="0.9"/>
<g transform="translate(740 40) scale(4.4)">{MARK}</g>
<text x="60" y="92" font-family="'JetBrains Mono','DejaVu Sans Mono',monospace" font-size="20" letter-spacing="3" fill="#ff793f">{tag}</text>
<text x="56" y="176" font-family="Inter,'Helvetica Neue',Arial,sans-serif" font-weight="800" font-size="84" letter-spacing="-4" fill="#f2ede5">{title}</text>
<text x="60" y="226" font-family="'JetBrains Mono','DejaVu Sans Mono',monospace" font-size="22" fill="#b8afa5">{sub}</text>
<rect x="60" y="246" width="120" height="4" fill="#ff793f"/>
</svg>"""
    open(f"{name}.svg", "w").write(svg)
    subprocess.run(["rsvg-convert", "-w", str(w), f"{name}.svg", "-o", f"{name}.png"], check=True)

banner("ticket", "SUPPORT", "open a private ticket - staff will reply", "LOKI DEV / TICKETS")
banner("verify", "VERIFY", "react to unlock the server", "LOKI DEV / ACCESS")
banner("welcome", "WELCOME", "good to have you here", "LOKI DEV / COMMUNITY")
banner("help", "COMMANDS", "everything the bot can do", "LOKI DEV / HELP")


def poster(name, title, sub, tag):
    """Vertical 3:4 poster for help category pages; title size shrinks to fit the width."""
    w, h = 900, 1200
    size = min(150, int(760 / (0.66 * len(title))))
    defs = DEFS.replace('"W"', f'"{w}"').replace('"H"', f'"{h}"').replace('cx="82%" cy="50%"', 'cx="50%" cy="36%"')
    svg = f"""<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}">
{defs}
<rect width="{w}" height="{h}" fill="#100f0e"/>
<rect width="{w}" height="{h}" fill="url(#dots)" mask="url(#halo)" opacity="0.9"/>
<g transform="translate(258 180) scale(8)">{MARK}</g>
<text x="70" y="860" font-family="'JetBrains Mono','DejaVu Sans Mono',monospace" font-size="26" letter-spacing="4" fill="#ff793f">{tag}</text>
<text x="64" y="{880 + size}" font-family="Inter,'Helvetica Neue',Arial,sans-serif" font-weight="800" font-size="{size}" letter-spacing="-4" fill="#f2ede5">{title}</text>
<text x="70" y="1100" font-family="'JetBrains Mono','DejaVu Sans Mono',monospace" font-size="28" fill="#b8afa5">{sub}</text>
<rect x="70" y="1130" width="160" height="6" fill="#ff793f"/>
</svg>"""
    open(f"{name}.svg", "w").write(svg)
    subprocess.run(["rsvg-convert", "-w", str(w), f"{name}.svg", "-o", f"{name}.png"], check=True)

poster("help-information", "INFORMATION", "server &amp; bot info", "LOKI DEV / INFORMATION")
poster("help-moderation", "MODERATION", "mod tools - requires perms", "LOKI DEV / MODERATION")
poster("help-tickets", "TICKETS", "support tickets &amp; blacklist", "LOKI DEV / TICKETS")
poster("help-utility", "UTILITY", "polls, reminders, blog", "LOKI DEV / UTILITY")

avatar = f"""<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" width="1024" height="1024">
<defs>
  <linearGradient id="bg" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#ff9a5d"/><stop offset="1" stop-color="#ff5a1f"/></linearGradient>
  <mask id="cut"><rect width="48" height="48" fill="#fff"/><path d="M24 2V46" stroke="#000" stroke-width="3"/>
    <path d="M16 22l8 4 8-4" stroke="#000" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"/></mask>
  <pattern id="dots" width="12" height="12" patternUnits="userSpaceOnUse"><rect x="1" y="1" width="5" height="5" fill="#ffffff" fill-opacity="0.16"/><rect x="7" y="7" width="3" height="3" fill="#ffffff" fill-opacity="0.16"/></pattern>
</defs>
<rect width="512" height="512" fill="url(#bg)"/>
<rect width="512" height="512" fill="url(#dots)"/>
<g transform="translate(88 92) scale(7)"><path d="M24 4l18 20-18 20L6 24z" fill="#100f0e" mask="url(#cut)"/></g>
</svg>"""
open("avatar.svg", "w").write(avatar)
subprocess.run(["rsvg-convert", "-w", "1024", "avatar.svg", "-o", "avatar.png"], check=True)
