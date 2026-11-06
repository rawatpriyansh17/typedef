const modes = {
  time: ['◷','You vs. the clock.','15, 30, 60 or 120 seconds. Your first key starts the timer. Chase speed without losing accuracy.','tpdf --mode time --seconds 30'],
  words: ['≡','Set your distance.','Choose a word count and take your time. The test ends when you reach the final character.','tpdf --mode words --words 50'],
  quote: ['“','A little more character.','Practice complete passages with punctuation and capitals. Finish the quote to see your results.','tpdf --mode quote'],
  endless: ['∞','No finish line.','Fresh words keep arriving. Follow your live stats and 50-word laps. Ctrl+D finishes and saves your run.','tpdf --mode endless'],
  zen: ['◇','Turn down the noise.','Unlimited guided words in focus mode. No countdown or word limit. Ctrl+D finishes and saves.','tpdf --mode zen'],
  daily: ['◈','Same day. Same challenge.','A 60-second test shared by UTC date and app version. A daily ritual, with no account required.','tpdf --mode daily'],
  weak: ['⌁','Work on the tricky keys.','Train the characters you miss most. Your local error history selects words worth practicing.','tpdf --mode weak'],
  custom: ['↥','Bring your own words.','Press O in the app to browse or paste a local file path. Text, notes, or source code. Nothing is uploaded.','tpdf --file notes.txt'],
  code: ['{}','Keep the indentation.','Type bundled Rust, Python and JavaScript snippets. Enter adds a newline; Tab fills leading indentation.','tpdf --mode code']
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
const palettes = {fern:['Fern','01'],cobalt:['Cobalt','14'],rose:['Rose','15'],paper:['Paper','02']};
document.querySelectorAll('[data-theme]').forEach(button => button.addEventListener('click', () => {
  document.querySelectorAll('[data-theme]').forEach(b => b.setAttribute('aria-pressed',String(b === button)));
  const key = button.dataset.theme, [name,number] = palettes[key];
  const image = document.querySelector('#theme-image');
  image.src = `/assets/${key}.webp`;
  image.alt = `typedef dashboard in the ${name} theme`;
  document.querySelector('#theme-caption').textContent = `${name.toUpperCase()} / ${number} — Actual app capture`;
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
