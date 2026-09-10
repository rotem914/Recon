// S0.4's evidence, run with ?selftest=1 in the editor window.
//
// The plan asks this step to show every callout operation working, an empty bubble
// discarded, that a hidden pre-created window shows a current first frame, and that the
// detail on screen matches the zoom. Each of those is asserted here rather than looked at,
// because a check that needs someone to squint is a check that stops being run.
//
// The two that matter most are the last two. "Zoom cannot re-wrap text" is the property
// part 5 claims by construction, so it is worth a test that would fail if the layout ever
// moved inside the transform. And "the detail matches the zoom" is F35: a preview made for
// fit-to-window, enlarged, is mush, and the probe image makes that visible in the pixels.

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

export async function runChecks(editor, invoke) {
  const lines = [];
  let failures = 0;

  const report = document.getElementById('report');
  document.body.classList.add('reporting');
  const say = (text) => {
    lines.push(text);
    report.textContent = lines.join('\n');
  };
  const ok = (name, detail = '') => say(`  pass  ${name}${detail ? '   ' + detail : ''}`);
  const bad = (name, detail) => { failures += 1; say(`  FAIL  ${name}   ${detail}`); };
  const check = (name, condition, detail = '') => condition ? ok(name, detail) : bad(name, detail);

  const { model } = editor;
  const el = (callout) => document.querySelector(`[data-id="${callout.id}"]`);

  say('=== S0.4: the editor window, the scene, one callout ===');
  say(`image ${model.image.width}x${model.image.height} from ${model.image.source}`);
  say(`physical-to-css ratio ${editor.ratioOf().toFixed(3)}, which devicePixelRatio reports as ${window.devicePixelRatio}`);
  say('');

  // ---------------------------------------------------------------- 1. the callout, end to end
  say('the callout, every operation the step names');
  {
    const before = model.callouts.length;
    const callout = editor.createCallout({ x: 400, y: 300 });
    editor.layoutScene();
    check('created by a click on the image', model.callouts.length === before + 1);
    check('anchored where the click was', callout.anchor.x === 400 && callout.anchor.y === 300);
    check('a bubble exists in the document', !!el(callout));

    editor.startEditing(callout);
    const textEl = el(callout).querySelector('.t');
    textEl.textContent = 'the save button does nothing on a slow connection';
    editor.commitEditing();
    check('typed text survives the commit', callout.text.startsWith('the save button'), JSON.stringify(callout.text.slice(0, 24)));
    check('editing has ended', model.editing === null);

    model.selected = callout;
    editor.layoutScene();
    check('selected', el(callout).classList.contains('selected'));

    const box0 = { ...callout.box };
    callout.box.x += 90;
    callout.box.y += 40;
    editor.layoutScene();
    check('the bubble moves', callout.box.x === box0.x + 90 && el(callout).style.left === `${box0.x + 90}px`);
    check('moving the bubble leaves the anchor alone', callout.anchor.x === 400 && callout.anchor.y === 300);

    const line0 = document.querySelector('#arrows line');
    const end0 = line0 ? { x: Number(line0.getAttribute('x2')), y: Number(line0.getAttribute('y2')) } : null;
    callout.anchor.x = 700;
    callout.anchor.y = 520;
    editor.layoutScene();
    const line1 = document.querySelector('#arrows line');
    const start1 = { x: Number(line1.getAttribute('x1')), y: Number(line1.getAttribute('y1')) };
    const end1 = { x: Number(line1.getAttribute('x2')), y: Number(line1.getAttribute('y2')) };
    check('the anchor moves on its own', start1.x === 700 && start1.y === 520);
    check('the arrow follows it', !!end0 && (end1.x !== end0.x || end1.y !== end0.y),
      `ends at ${end1.x},${end1.y}`);

    editor.removeCallout(callout);
    editor.layoutScene();
    check('deleted, and gone from the document', model.callouts.length === before && !el(callout));
  }

  // ---------------------------------------------------------------- 2. the empty bubble
  say('');
  say('an empty bubble never reaches the output');
  {
    const before = model.callouts.length;
    const callout = editor.createCallout({ x: 200, y: 200 });
    editor.layoutScene();
    editor.startEditing(callout);
    editor.commitEditing();
    check('discarded when editing ends with no text', model.callouts.length === before);

    const spaces = editor.createCallout({ x: 220, y: 220 });
    editor.layoutScene();
    editor.startEditing(spaces);
    el(spaces).querySelector('.t').textContent = '   ';
    editor.commitEditing();
    check('whitespace counts as empty', model.callouts.length === before);
  }

  // ---------------------------------------------------------------- 3. numbering keeps its gaps
  say('');
  say('numbering, which §3.5 says keeps its gaps everywhere');
  {
    model.callouts = [];
    model.nextNumber = 1;
    const a = editor.createCallout({ x: 100, y: 100 });
    const b = editor.createCallout({ x: 200, y: 100 });
    const c = editor.createCallout({ x: 300, y: 100 });
    a.text = 'one'; b.text = 'two'; c.text = 'three';
    editor.layoutScene();
    check('numbered by creation order', a.number === 1 && b.number === 2 && c.number === 3);
    editor.removeCallout(b);
    const d = editor.createCallout({ x: 400, y: 100 });
    d.text = 'four';
    editor.layoutScene();
    check('a deleted number is not reused', d.number === 4, `after deleting 2, the next is ${d.number}`);
    check('the others are not renumbered', a.number === 1 && c.number === 3);
    model.callouts = [];
    editor.layoutScene();
  }

  // ---------------------------------------------------------------- 4. direction, three modes
  say('');
  say('text direction: three explicit modes, resolved once per bubble');
  {
    const hebrew = editor.createCallout({ x: 100, y: 100 });
    hebrew.text = 'שלום, this line starts in Hebrew';
    const english = editor.createCallout({ x: 100, y: 200 });
    english.text = 'This line starts in English, ואז עברית';
    const overridden = editor.createCallout({ x: 100, y: 300 });
    overridden.text = 'This line starts in English';
    overridden.dirMode = 'rtl';
    editor.layoutScene();
    check('auto resolves a Hebrew opening to rtl', editor.resolveDirection(hebrew) === 'rtl');
    check('auto resolves an English opening to ltr', editor.resolveDirection(english) === 'ltr');
    check('an override wins over the heuristic', editor.resolveDirection(overridden) === 'rtl');
    check('the resolved value reaches the element',
      el(overridden).style.direction === 'rtl' && el(english).style.direction === 'ltr');
    model.callouts = [];
    editor.layoutScene();
  }

  // ---------------------------------------------------------------- 5. zoom cannot re-wrap
  say('');
  say('zoom is a transform, so it cannot re-wrap a line (part 5, by construction)');
  {
    const callout = editor.createCallout({ x: 300, y: 300 });
    callout.text =
      'A long note that has to wrap over several lines so that any change in the wrapping ' +
      'would move its height, in Hebrew and English together: שורה ארוכה שנשברת לכמה שורות.';
    editor.layoutScene();
    const box = el(callout);
    const heights = [];
    for (const zoom of [model.fitZoom, 1, 2, model.fitZoom / 2, model.fitZoom]) {
      await editor.setZoom(zoom);
      await sleep(60);
      heights.push({ zoom: Number(model.zoom.toFixed(4)), height: box.offsetHeight, lines: box.querySelector('.t').getClientRects().length });
    }
    const first = heights[0].height;
    const same = heights.every((h) => h.height === first);
    check('the laid-out height is identical at every zoom', same,
      heights.map((h) => `${(h.zoom * 100).toFixed(0)}%:${h.height}px`).join('  '));
    model.callouts = [];
    editor.layoutScene();
  }

  // ---------------------------------------------------------------- 6. the detail matches the zoom
  say('');
  say('the detail matches the zoom, which is F35 and the reason the proxy has a policy');
  {
    const info = await invoke('editor_load_probe', { width: 3840, height: 2160 });
    await editor.loadImage(info);

    // The probe's LEFT HALF is a one-pixel checkerboard and its right half is something
    // else, so the strip has to be read at a known place in the IMAGE rather than at a
    // fixed place on the canvas. Reading canvas x=0 after a pan was the first version's
    // mistake: it wandered into the other half of the probe and called it mush.
    const crispness = () => {
      const canvasX = Math.max(0, Math.round((300 - model.pan.x) * model.zoom));
      const width = Math.min(200, Math.max(8, editor.canvas.width - canvasX - 1));
      const y = Math.floor(editor.canvas.height / 4);
      const strip = editor.context.getImageData(canvasX, y, width, 1).data;
      let flips = 0;
      let extremes = 0;
      for (let i = 4; i < strip.length; i += 4) {
        if (Math.abs(strip[i] - strip[i - 4]) > 100) flips += 1;
        if (strip[i] < 24 || strip[i] > 231) extremes += 1;
      }
      return { flips, extremes, samples: strip.length / 4, canvasX };
    };

    await editor.setZoom(1);
    const actual = crispness();
    check('at actual size the one-pixel checkerboard is still a checkerboard',
      actual.flips > actual.samples * 0.6,
      `${actual.flips} flips over ${actual.samples} pixels, ${actual.extremes} at the extremes`);
    const stage = document.getElementById('stage');
    const expected = Math.round(stage.clientWidth * editor.ratioOf());
    check('at actual size the canvas backing store is one pixel per image pixel',
      Math.abs(editor.canvas.width - expected) <= 1,
      `canvas ${editor.canvas.width}x${editor.canvas.height}, expected ${expected} wide at ratio ${editor.ratioOf().toFixed(3)}`);

    await editor.setZoom(model.fitZoom);
    const fitted = crispness();
    check('at fit-to-window the same pixels are averaged, not alternating',
      fitted.flips < actual.flips / 3,
      `${fitted.flips} flips against ${actual.flips} at actual size`);

    await editor.setZoom(1);
    const again = crispness();
    check('going back to actual size is crisp again, not an enlarged preview',
      again.flips > again.samples * 0.6,
      `${again.flips} flips`);

    // And after a pan, still inside the checkerboard half, because the point is the policy
    // and not the probe's layout.
    model.pan.x += 200;
    await editor.paintRegion();
    const panned = crispness();
    check('still crisp after a pan', panned.flips > panned.samples * 0.6, `${panned.flips} flips`);
  }

  // ---------------------------------------------------------------- 7. the first frame after a show
  say('');
  say('the hidden window shows a CURRENT first frame, not a blank or stale one');
  {
    // Painted while the window is still hidden, then the host shows it and looks at the
    // screen. Asking the page what it drew would prove nothing: a suspended surface has a
    // perfectly correct document behind it.
    // The whole page becomes the colour, and both overlays come off, because the first
    // version painted the canvas and then sampled the report panel sitting on top of it:
    // 0% matching and 0% black, which is the screen honestly reporting a third thing.
    const colour = { r: 0, g: 160, b: 90 };
    const restore = { body: document.body.style.background, cls: document.body.className };
    document.body.className = '';
    document.body.style.background = `rgb(${colour.r},${colour.g},${colour.b})`;
    document.getElementById('stage').style.visibility = 'hidden';
    document.getElementById('hud').style.visibility = 'hidden';
    await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));
    await sleep(60);

    const looks = await invoke('editor_show_and_look', colour);
    document.body.style.background = restore.body;
    document.body.className = restore.cls;
    document.getElementById('stage').style.visibility = '';
    document.getElementById('hud').style.visibility = '';
    for (const look of looks) {
      const good = look.matching > 0.9 && look.black < 0.05;
      check(`${look.label}: the screen shows what the page painted`, good,
        `${(look.matching * 100).toFixed(1)}% matching, ${(look.black * 100).toFixed(1)}% black, ${look.sampled} sampled, ${look.rect}`);
      say(`        just outside the reported right edge: ${(look.outside_matching * 100).toFixed(1)}% the same colour` +
        ` (high would mean the window is physically bigger than the size the runtime reports)`);
    }
    await editor.paintRegion();
  }

  say('');
  if (failures === 0) {
    say('RESULT: every check passed.');
  } else {
    say(`RESULT: ${failures} check(s) failed.`);
  }
  await invoke('editor_checks_done', { report: lines.join('\n'), failures });
}
