const modes = {
  time: ['◷','Timed practice','Choose 15, 30, 60 or 120 seconds. Your first key starts the timer.','tpdf --mode time --seconds 30'],
  words: ['≡','Word count','Choose a word count and take your time. The test ends when you reach the final character.','tpdf --mode words --words 50'],
  quote: ['“','Quotes','Practice complete passages with punctuation and capitals. Finish the quote to see your results.','tpdf --mode quote'],
  endless: ['∞','Endless practice','Fresh words keep arriving. Follow your live stats and 50-word laps. Ctrl+D finishes and saves your run.','tpdf --mode endless'],
  zen: ['◇','Focus mode','Unlimited guided words in focus mode. No countdown or word limit. Ctrl+D finishes and saves.','tpdf --mode zen'],
  daily: ['◈','Daily challenge','A 60-second test with the same starting words for everyone using the same app version that day.','tpdf --mode daily'],
  weak: ['⌁','Weak keys','Train the characters you miss most. Your local error history selects words worth practicing.','tpdf --mode weak'],
  custom: ['↥','Your own text','Press O in the app to browse or paste a local file path. Text, notes, or source code. Nothing is uploaded.','tpdf --file notes.txt'],
  code: ['{}','Code practice','Type bundled Rust, Python and JavaScript snippets. Enter adds a newline; Tab fills leading indentation.','tpdf --mode code']
};
const tabs = [...document.querySelectorAll('[data-mode]')];
function selectMode(tab) {
  tabs.forEach(item => { item.setAttribute('aria-selected', String(item === tab)); item.tabIndex = item === tab ? 0 : -1; });
  const [symbol,title,description,command] = modes[tab.dataset.mode];
  document.querySelector('#mode-symbol').textContent = symbol;
  document.querySelector('#mode-title').textContent = title;
  document.querySelector('#mode-description').textContent = description;
  document.querySelector('#mode-command').textContent = command;
  document.querySelector('#mode-panel').setAttribute('aria-labelledby', tab.id);
}
tabs.forEach((tab,index) => {
  tab.addEventListener('click', () => selectMode(tab));
  tab.addEventListener('keydown', event => {
    let next;
    if (event.key === 'ArrowRight' || event.key === 'ArrowDown') next = (index+1)%tabs.length;
    if (event.key === 'ArrowLeft' || event.key === 'ArrowUp') next = (index+tabs.length-1)%tabs.length;
    if (event.key === 'Home') next = 0;
    if (event.key === 'End') next = tabs.length-1;
    if (next !== undefined) { event.preventDefault(); selectMode(tabs[next]); tabs[next].focus(); }
  });
});
const palettes = {fern:['Fern','01'],cobalt:['Cobalt','14'],rose:['Rose','15'],ember:['Ember','06'],paper:['Paper','02']};
const themeButtons = [...document.querySelectorAll('[data-theme]')];
function applyTheme(key, remember = false) {
  if (!Object.hasOwn(palettes, key)) key = 'fern';
  document.documentElement.dataset.palette = key;
  themeButtons.forEach(button => button.setAttribute('aria-pressed', String(button.dataset.theme === key)));
  const [name] = palettes[key];
  const image = document.querySelector('#theme-image');
  image.src = `/assets/${key}.webp`;
  image.alt = `typedef dashboard in the ${name} theme`;
  document.querySelector('#theme-caption').textContent = `${name} · app capture`;
  document.querySelector('meta[name="theme-color"]').content = getComputedStyle(document.documentElement).getPropertyValue('--bg').trim();
  if (remember) {
    try { localStorage.setItem('typedef-theme', key); } catch { /* Storage may be disabled. */ }
  }
}
themeButtons.forEach(button => button.addEventListener('click', () => applyTheme(button.dataset.theme, true)));
let savedTheme = 'fern';
try { savedTheme = localStorage.getItem('typedef-theme') || savedTheme; } catch { /* Use the default palette. */ }
applyTheme(savedTheme);
const installMethods = {
  curl: ['curl -fsSL https://typedef-tui.vercel.app/install.sh | sh', 'macOS + Linux', 'Verifies SHA-256 and installs to ~/.local/bin. No sudo or shell profile changes.'],
  brew: ['brew install rawatpriyansh17/tap/typedef', 'macOS + Linux · Homebrew', 'Installs both commands. Update later with brew upgrade typedef.'],
  cargo: ['cargo install --git https://github.com/rawatpriyansh17/typedef --locked', 'All platforms · Rust required', 'Builds from source with Cargo. Use curl or Homebrew for prebuilt binaries.']
};
document.querySelectorAll('[data-install]').forEach(button => button.addEventListener('click', () => {
  document.querySelectorAll('[data-install]').forEach(b => b.setAttribute('aria-pressed', String(b === button)));
  const [command,platform,note] = installMethods[button.dataset.install];
  document.querySelector('#install-command').textContent = command;
  document.querySelector('#install-platform').textContent = platform;
  document.querySelector('#install-note').textContent = note;
  document.querySelector('#copy-status').textContent = '';
}));
document.querySelector('#copy-command').addEventListener('click', async () => {
  const command = document.querySelector('#install-command').textContent;
  try {
    await navigator.clipboard.writeText(command);
    document.querySelector('#copy-status').textContent = 'Copied. Paste it into your terminal.';
  } catch {
    const range = document.createRange(); range.selectNodeContents(document.querySelector('#install-command'));
    const selection = window.getSelection(); selection.removeAllRanges(); selection.addRange(range);
    document.querySelector('#copy-status').textContent = 'Command selected. Press Ctrl+C or ⌘C to copy.';
  }
});
const video = document.querySelector('#demo-video');
const toggle = document.querySelector('#motion-toggle');
const reducedMotion = matchMedia('(prefers-reduced-motion: reduce)');
let manualPause = false;
function syncVideoButton() { toggle.textContent = video.paused ? 'Play demo ▷' : 'Pause demo Ⅱ'; }
video.addEventListener('play',syncVideoButton); video.addEventListener('pause',syncVideoButton);
toggle.addEventListener('click', () => { if (video.paused) { manualPause=false; video.play().catch(syncVideoButton); } else { manualPause=true; video.pause(); } });
if ('IntersectionObserver' in window) {
  new IntersectionObserver(entries => {
    const visible = entries[0].isIntersecting;
    if (!visible) video.pause();
    else if (!manualPause && !reducedMotion.matches && !navigator.connection?.saveData) video.play().catch(syncVideoButton);
  }, {threshold:.25}).observe(video);
}
reducedMotion.addEventListener('change', () => { if (reducedMotion.matches) video.pause(); });
