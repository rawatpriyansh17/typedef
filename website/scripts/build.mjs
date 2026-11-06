import {mkdir,rm,cp} from 'node:fs/promises';
await rm('dist',{recursive:true,force:true});
await mkdir('dist');
for(const path of ['index.html','style.css','app.js','assets']) await cp(path,`dist/${path}`,{recursive:true});
console.log('Built static site in website/dist');
