"""Render genuine Ratatui capture frames to website images and an MP4.
First: TYPEDEF_SNAPSHOT=1 cargo test --lib capture_site_media -- --ignored
Requires Pillow and FFmpeg (or imageio-ffmpeg). No user profile is accessed.
"""
import json, re, os, shutil, subprocess, tempfile
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont
ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'website/assets'
font_path = os.environ.get('TYPEDEF_FONT') or next(str(p) for p in [Path('/System/Library/Fonts/Menlo.ttc'),Path('/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf')] if p.exists())
font = ImageFont.truetype(font_path, 15)
COLORS = {'Reset': (215,226,225), 'Green':(100,200,140),'Cyan':(120,210,220),'Red':(245,120,130),'DarkGray':(110,120,130),'Black':(0,0,0)}
def color(s,default):
    m = re.fullmatch(r'Rgb\((\d+), (\d+), (\d+)\)',s)
    return tuple(map(int,m.groups())) if m else COLORS.get(s,default)
def render(path):
    data=json.loads(path.read_text()); w,h=data['width'],data['height']
    im=Image.new('RGB',(w*10,h*20)); d=ImageDraw.Draw(im)
    for i,c in enumerate(data['cells']):
        x,y=i%w*10,i//w*20; fg=color(c['fg'],(215,226,225)); bg=color(c['bg'],(16,23,29))
        d.rectangle((x,y,x+9,y+19),fill=bg)
        ch=c['s']
        if ch=='█': d.rectangle((x,y,x+9,y+19),fill=fg)
        elif ch=='▀': d.rectangle((x,y,x+9,y+9),fill=fg)
        elif ch=='▄': d.rectangle((x,y+10,x+9,y+19),fill=fg)
        elif len(ch)==1 and 0x2800<=ord(ch)<=0x28ff:
            bits=ord(ch)-0x2800
            for bit,dx,dy in [(0,1,1),(1,1,6),(2,1,11),(6,1,16),(3,6,1),(4,6,6),(5,6,11),(7,6,16)]:
                if bits & (1<<bit): d.ellipse((x+dx,y+dy,x+dx+1,y+dy+1),fill=fg)
        else: d.text((x,y),ch,font=font,fill=fg)
    return im
OUT.mkdir(exist_ok=True)
for name in ['home','typing','results','journey','fern','cobalt','rose','paper']:
    render(Path('/tmp/typedef-site-'+name+'.json')).save(OUT/(name+'.webp'), quality=92)
render(Path('/tmp/typedef-site-home.json')).save(OUT/'dashboard.png',optimize=True)
ffmpeg = shutil.which('ffmpeg')
if not ffmpeg:
    import imageio_ffmpeg
    ffmpeg=imageio_ffmpeg.get_ffmpeg_exe()
with tempfile.TemporaryDirectory(prefix='typedef-video-') as tmp:
    for i in range(216):
        render(Path(f'/tmp/typedef-video-{i:04}.json')).save(Path(tmp)/f'{i:04}.png')
    subprocess.run([ffmpeg,'-hide_banner','-loglevel','error','-y','-framerate','12','-i',str(Path(tmp)/'%04d.png'),'-c:v','libx264','-crf','23','-pix_fmt','yuv420p','-movflags','+faststart','-an',str(OUT/'demo.mp4')],check=True)
print('Wrote screenshots and 18-second app demo to',OUT)
