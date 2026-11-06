# typedef website

A static landing page for the terminal app, maintained in the same repository. Plain HTML, CSS and JavaScript; no runtime dependencies, external fonts, analytics, account service or backend. The Rust app does not need Node.js.

## Deploy on Vercel

1. Import `rawatpriyansh17/typedef` as a new Vercel project.
2. Set **Root Directory** to **`website`**.
3. Select **Other** as the framework preset.
4. Use **`npm run build`** as the build command and **`dist`** as the output directory. These are also specified in `vercel.json`.
5. Deploy. No environment variables are required. Add your domain in Vercel when ready.

This creates a website deployment without building Rust. Future pushes to `main` can update the production website; pull requests can receive Vercel previews. A separate repository is unnecessary. See [Vercel's monorepo documentation](https://vercel.com/docs/monorepos) for Root Directory configuration.

The repository includes deployment configuration, not a provisioned Vercel project. After choosing a public domain, make the Open Graph image URL in `index.html` absolute for reliable social previews.

## Local development

Requires Node.js 20+ and Python 3:

```sh
cd website
npm ci
npm run dev
# Open http://localhost:8000
```

```sh
npm run build
python3 -m http.server 8000 --directory dist
```

The build checks JavaScript syntax, local assets, mode controls and media, then copies only public files into `dist/`. Generated output is ignored by Git. No application or profile data is included.

## App media

The screenshots and 18-second MP4 use the actual Ratatui renderer at 120×36 cells. Input and sample history are automated fixtures, not a real user's scores. No private profile is accessed. The video is silent; playback pauses offscreen and respects reduced-motion/data-saving preferences. Native controls and an explicit play/pause button remain available.

To regenerate after changing the terminal UI, from the repository root:

```sh
TYPEDEF_SNAPSHOT=1 cargo test --lib capture_site_media -- --ignored
python3 -m venv /tmp/typedef-media-env
/tmp/typedef-media-env/bin/pip install Pillow imageio-ffmpeg
/tmp/typedef-media-env/bin/python scripts/render_media.py
```

The renderer uses Menlo on macOS or DejaVu Sans Mono on Linux. Set `TYPEDEF_FONT` to another monospace font path if needed. FFmpeg can be installed separately instead of using `imageio-ffmpeg`. Capture JSON is temporary under `/tmp`; final WebP/PNG/MP4 assets are committed in `assets/` so deployment needs no Rust, Python or video encoder.

## Editing

- `index.html`: content, landmarks, installation instructions and real repository links.
- `style.css`: responsive layout, terminal-inspired artwork and reduced-motion rules.
- `app.js`: accessible mode tabs, theme previews, copy action and video behavior.
- `assets/`: original logo and genuine app captures.
- `scripts/`: dependency-free validation and build.
- `vercel.json`: static build and response headers.

Keep the nine mode descriptions and install command in sync with the app. Verify keyboard tab navigation, all theme previews, copy feedback, video controls, and 320px/390px/desktop layouts. Product presentation references were [Ghostty](https://ghostty.org/) and [Zed](https://zed.dev/): restrained navigation, strong typography and visible product demos. No site code or artwork was copied.
