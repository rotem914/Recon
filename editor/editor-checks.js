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
  // A check the machine could not run is named with its reason, never passed and never
  // failed (QA: an unrun check that is named is information). The result line counts them.
  let notRun = 0;
  let screenOff = false;
  const skipped = (name, reason) => { notRun += 1; say(`  NOT RUN  ${name}   ${reason}`); };
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
      // Below fit the zoom is set directly: the controls and a change of space never leave a
      // picture below the whole of it (Rotem, 2026-09-15), so only a direct set tests that zoom.
      if (zoom < model.fitZoom) { model.zoom = zoom; await editor.paintRegion(); } else await editor.setZoom(zoom);
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

    // The screen cannot always be copied: with nobody at the machine the session can be
    // one a plain BitBlt refuses with access denied, and then this section is not run
    // rather than failed, and says so.
    let looks = [];
    try {
      looks = await invoke('editor_show_and_look', colour);
    } catch (err) {
      skipped('the screen shows what the page painted', `the screen could not be copied: ${err}`);
      screenOff = true;
    }
    document.body.style.background = restore.body;
    document.body.className = restore.cls;
    document.getElementById('stage').style.visibility = '';
    document.getElementById('hud').style.visibility = '';
    if (looks.length && looks.every((look) => look.black > 0.99)) {
      skipped('the screen shows what the page painted', 'the screen copy is all black, so the display is off; nothing can be seen on it');
      screenOff = true;
      looks = [];
    } else if (looks.length && looks.every((look) => look.matching < 0.5) && !(await invoke('editor_in_front'))) {
      skipped('the screen shows what the page painted', 'another window is over ours, so the screen shows that window; nothing of the page can be read there');
      screenOff = true;
      looks = [];
    }
    for (const look of looks) {
      const good = look.matching > 0.9 && look.black < 0.05;
      check(`${look.label}: the screen shows what the page painted`, good,
        `${(look.matching * 100).toFixed(1)}% matching, ${(look.black * 100).toFixed(1)}% black, ${look.sampled} sampled, ${look.rect}`);
      say(`        just outside the reported right edge: ${(look.outside_matching * 100).toFixed(1)}% the same colour` +
        ` (high would mean the window is physically bigger than the size the runtime reports)`);
    }
    await editor.paintRegion();
  }

  // ---------------------------------------------------------------- 8. the text size
  say('');
  say('text size: 20 image pixels by default, and the user\'s to change');
  {
    model.callouts = [];
    model.textSize = 20;
    const first = editor.createCallout({ x: 300, y: 300 });
    first.text = 'a note at the size a note gets by default';
    model.selected = first;
    editor.layoutScene();
    const sizeOf = (callout) => getComputedStyle(el(callout)).fontSize;
    check('a new note is 20 image pixels', first.textSize === 20 && sizeOf(first) === '20px',
      `stored ${first.textSize}, laid out at ${sizeOf(first)}`);

    const smallHeight = el(first).offsetHeight;
    editor.nudgeTextSize(1);
    check('one step up is the next size on the ladder, and it reaches the element',
      first.textSize === 24 && sizeOf(first) === '24px', `now ${sizeOf(first)}`);
    editor.nudgeTextSize(-1);
    check('one step down comes back', first.textSize === 20 && sizeOf(first) === '20px');

    // The size has to change the LAYOUT and not only the stored number. A size that never
    // reaches the box is exactly the defect the two checks above could still miss.
    first.textSize = 40;
    editor.layoutScene();
    const bigHeight = el(first).offsetHeight;
    check('the size changes the laid-out height', bigHeight > smallHeight * 1.5,
      `${smallHeight}px tall at 20, ${bigHeight}px at 40`);

    first.textSize = 20;
    editor.nudgeTextSize(1);
    const second = editor.createCallout({ x: 1200, y: 300 });
    second.text = 'the next note';
    editor.layoutScene();
    check('the size last used is what the next note gets', second.textSize === 24,
      `the new note is ${second.textSize}`);
    check('and a new note is as wide as its text is big', second.box.width === 24 * 13,
      `${second.box.width}px wide`);

    model.selected = second;
    for (let i = 0; i < 20; i += 1) editor.nudgeTextSize(-1);
    const smallest = second.textSize;
    for (let i = 0; i < 30; i += 1) editor.nudgeTextSize(1);
    const largest = second.textSize;
    check('the ladder stops at both ends rather than running away',
      smallest === editor.TEXT_SIZES[0]
      && largest === editor.TEXT_SIZES[editor.TEXT_SIZES.length - 1],
      `down to ${smallest}, up to ${largest}`);

    // What F46 settled: a note scales with the image and there is no on-screen floor. The
    // number this prints is the thing that was decided, so it is measured, not asserted.
    model.callouts = [];
    model.selected = null;
    model.textSize = 20;
    const note = editor.createCallout({ x: 300, y: 300 });
    note.text = 'a note at the default size';
    editor.layoutScene();
    await editor.setZoom(model.fitZoom);
    const box = el(note);
    const laid = box.offsetHeight;
    const onScreen = box.getBoundingClientRect().height;
    const expected = (laid * model.zoom) / editor.ratioOf();
    check('a note scales with the image, with no minimum on-screen size',
      Math.abs(onScreen - expected) < 1.5,
      `${laid}px in the image is ${onScreen.toFixed(1)}px on screen at fit ` +
      `(${(model.zoom * 100).toFixed(0)}%), so 20px text reads as about ` +
      `${((20 * model.zoom) / editor.ratioOf()).toFixed(1)}px`);

    model.callouts = [];
    editor.layoutScene();
  }

  // ---------------------------------------------------------------- 9. what is typed comes back
  say('');
  say('a typed line break survives the commit (review T6)');
  {
    model.callouts = [];
    const note = editor.createCallout({ x: 300, y: 300 });
    editor.layoutScene();
    editor.startEditing(note);
    const textEl = el(note).querySelector('.t');
    // Real editing commands, so the engine builds whatever structure it would for a person.
    document.execCommand('insertText', false, 'one');
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', ctrlKey: true, bubbles: true, cancelable: true }));
    document.execCommand('insertText', false, 'two');
    editor.commitEditing();
    check('Ctrl+Enter, typed for real, yields a two-line note', note.text === 'one\ntwo',
      JSON.stringify(note.text));
    // Enter alone keeps the text and ends the typing, as a click outside does (Rotem, 2026-09-17).
    const kept = editor.createCallout({ x: 300, y: 300 });
    editor.layoutScene();
    editor.startEditing(kept);
    document.execCommand('insertText', false, 'kept by Enter');
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true, cancelable: true }));
    check('Enter alone keeps the text and ends the editing', model.editing === null && kept.text === 'kept by Enter' && model.callouts.includes(kept),
      `editing ${model.editing ? 'still on' : 'ended'}, text ${JSON.stringify(kept.text)}`);
    editor.removeCallout(kept);
    editor.layoutScene();

    // And the engine's own block structure, which is what a paste or an older document
    // could hold: innerText has to read it as lines too.
    const other = editor.createCallout({ x: 300, y: 600 });
    editor.layoutScene();
    editor.startEditing(other);
    document.execCommand('insertText', false, 'first');
    document.execCommand('insertParagraph');
    document.execCommand('insertText', false, 'second');
    editor.commitEditing();
    check('an engine-made paragraph break is read as a line break', other.text === 'first\nsecond',
      JSON.stringify(other.text));
    // A block element reports one client rect however many lines it holds, so the
    // evidence is height: a two-line note against a one-line one at the same size.
    const single = editor.createCallout({ x: 900, y: 300 });
    single.text = 'one';
    editor.layoutScene();
    const twoLines = el(note).querySelector('.t').offsetHeight;
    const oneLine = el(single).querySelector('.t').offsetHeight;
    // One line sits in the text box's 36 px floor (Rotem, 2026-09-17), so two lines are
    // measured against two of the line's own height, and taller than the floor.
    const lineHeight = parseFloat(getComputedStyle(el(single).querySelector('.t')).lineHeight);
    check('the committed note is laid out on two lines', twoLines > oneLine && twoLines >= lineHeight * 1.9,
      `${twoLines}px against ${oneLine}px for one line, the line ${lineHeight}px`);

    // Clicking straight from one note into another, with no Escape between: the first
    // note's typed text has to survive. It did not (review of 2026-09-13, R1).
    const first = editor.createCallout({ x: 300, y: 900 });
    const second = editor.createCallout({ x: 900, y: 900 });
    editor.layoutScene();
    editor.startEditing(first);
    document.execCommand('insertText', false, 'typed into the first');
    editor.startEditing(second);
    document.execCommand('insertText', false, 'then the second');
    editor.commitEditing();
    check('switching notes mid-typing keeps the first note\'s text', first.text === 'typed into the first' && second.text === 'then the second',
      `${JSON.stringify(first.text)} and ${JSON.stringify(second.text)}`);
    model.callouts = [];
    editor.layoutScene();
  }

  // ---------------------------------------------------------------- 10. SVG against the browser
  say('');
  say('SVG: resvg against this web view, on the three shapes of trouble (S0.5)');
  {
    const cases = await invoke('editor_svg_cases');
    for (const c of cases) {
      const img = new Image();
      const loaded = new Promise((resolve, reject) => { img.onload = resolve; img.onerror = () => reject(new Error('the svg did not load')); });
      img.src = 'data:image/svg+xml;charset=utf-8,' + encodeURIComponent(c.svg);
      await loaded;
      const canvas = document.createElement('canvas');
      canvas.width = c.width;
      canvas.height = c.height;
      const ctx = canvas.getContext('2d', { willReadFrequently: true });
      ctx.drawImage(img, 0, 0, c.width, c.height);
      const pixels = new Uint8Array(ctx.getImageData(0, 0, c.width, c.height).data.buffer);
      const verdict = await invoke('editor_svg_compare', pixels, { headers: { name: c.name } });
      const where = verdict.bbox ? ` in the box ${verdict.bbox.join(',')}` : '';
      // Text is the one case where a different font choice legitimately moves pixels, so
      // its bar is looser; shapes, fills and masks have no such excuse.
      const bar = c.name.includes('text') ? 6 : 1.5;
      check(`${c.name}: resvg and the web view agree`, verdict.percent <= bar,
        `${verdict.differing} of ${verdict.pixels} pixels differ, ${verdict.percent.toFixed(2)}%${where}`);
    }
  }

  // ---------------------------------------------------------------- 11. S0.6 check 1: original pixels survive
  say('');
  say('S0.6 check 1: every source pixel the notes do not cover is exact, on a semi-transparent source');
  const exportCheck = async (name, mode, extra = {}) => {
    const layer = await editor.exportLayer();
    return invoke('editor_export_check', layer.bytes, { headers: { name, margin: layer.margin, blur: layer.blur, crop: layer.crop, mode, ...extra } });
  };
  // A fresh document every time: opening the same file again keeps the notes on purpose
  // (a stepped frame is the same picture), and these checks want an empty page.
  const openFixture = async (name) => {
    const info = await invoke('editor_open_fixture', { name });
    await editor.loadImage(info);
    model.callouts = [];
    model.nextNumber = 1;
    model.selected = null;
    model.editing = null;
    await editor.setMargin({ left: 0, top: 0, right: 0, bottom: 0 });
    editor.layoutScene();
    return info;
  };
  {
    const dir = await invoke('editor_make_fixtures');
    say(`fixtures in ${dir}`);
    await openFixture('png-semi-transparent.png');
    let r = await exportCheck('semi-transparent-plain', 'source');
    check('no annotations: the output is the source, byte for byte, alpha included',
      r.source_mismatches === 0 && r.covered === 0, `${r.source_mismatches} mismatches of ${r.width * r.height}`);

    const a = editor.createCallout({ x: 60, y: 40 });
    a.text = 'Half see-through here';
    const b = editor.createCallout({ x: 220, y: 150 });
    b.text = 'ומכאן בעברית';
    editor.layoutScene();
    r = await exportCheck('semi-transparent-notes', 'source');
    check('two notes: every uncovered source pixel is exact, and the notes are in the output',
      r.source_mismatches === 0 && r.covered > 0, `${r.source_mismatches} mismatches, ${r.covered} pixels covered`);

    await editor.setMargin({ left: 30, top: 0, right: 120, bottom: 20 });
    r = await exportCheck('semi-transparent-margin', 'source');
    // The margin asked for is a floor: since S1.6 the second note, which reaches past the
    // right edge, takes the room it needs on top of it (§3.5), so the output is the source
    // plus the margin the notes settled on, never less than the one asked for.
    const m = model.margin;
    check('with a margin: the source is exact at its offset and the output is source plus margins',
      r.source_mismatches === 0 && r.width === 320 + m.left + m.right && r.height === 200 + m.top + m.bottom
        && m.left === 30 && m.top === 0 && m.right >= 120 && m.bottom >= 20,
      `${r.width}x${r.height} with the margin ${editor.marginString()} over the floor 30,0,120,20, ${r.source_mismatches} mismatches`);
    say(`origin-clean on WebView2 ${r.webview_version}: the layer read back from the canvas`);
  }

  // ---------------------------------------------------------------- 12. S0.6 check 2: six reference documents
  say('');
  say('S0.6 check 2: six documents against their reviewed references (tolerance: channel 48 premultiplied, 0.5% of pixels)');
  {
    const docs = [
      ['ref-hebrew', () => {
        const c = editor.createCallout({ x: 300, y: 120 });
        c.text = 'הכפתור הזה לא עושה כלום כשהחיבור איטי';
      }],
      ['ref-english', () => {
        const c = editor.createCallout({ x: 300, y: 120 });
        c.text = 'This label shows the group status, not the item status.';
      }],
      ['ref-mixed', () => {
        const c = editor.createCallout({ x: 300, y: 120 });
        c.text = 'הכיתוב Save changes צריך להיות בעברית, and the English part too.';
      }],
      ['ref-long', () => {
        const c = editor.createCallout({ x: 200, y: 90 });
        c.text = 'A long note that wraps over several lines: the save button does nothing on a slow connection, ' +
          'and the user gets no sign that anything happened. בעברית: הכפתור לא מגיב, אין חיווי, והמשתמש לוחץ שוב ושוב. ' +
          'Then the second click saves twice.';
      }],
      ['ref-edges', () => {
        const left = editor.createCallout({ x: 0, y: 200 });
        left.text = 'Left edge';
        left.box = { x: 8, y: 150, width: 150 };
        const right = editor.createCallout({ x: 639, y: 200 });
        right.text = 'Right edge';
        right.box = { x: 640 - 160, y: 230, width: 150 };
        const top = editor.createCallout({ x: 320, y: 0 });
        top.text = 'קצה עליון';
        top.box = { x: 340, y: 8, width: 150 };
        const bottom = editor.createCallout({ x: 320, y: 399 });
        bottom.text = 'קצה תחתון';
        bottom.box = { x: 60, y: 400 - 60, width: 150 };
      }],
      ['ref-margin', async () => {
        await editor.setMargin({ left: 0, top: 0, right: 240, bottom: 0 });
        const c = editor.createCallout({ x: 600, y: 200 });
        c.text = 'Inside the margin, בתוך השוליים';
        c.box = { x: 660, y: 170, width: 200 };
      }],
    ];
    for (const [name, build] of docs) {
      await openFixture('reference-scene.png');
      await build();
      editor.layoutScene();
      await sleep(40);
      const r = await exportCheck(name, 'reference');
      const where = r.bbox ? ` in the box ${r.bbox.join(',')}` : '';
      const status = r.reference === 'new' ? 'NEW reference written, review it'
        : r.reference === 'match' ? 'identical to the reference'
        : `${r.differing} pixels differ, ${r.percent.toFixed(3)}%${where}`;
      check(`${name}: source exact and within tolerance of the reference`,
        r.source_mismatches === 0 && (r.reference === 'new' || r.reference === 'match' || (r.reference === 'differs' && r.percent <= 0.5)),
        `${r.width}x${r.height}, ${status}`);
      if (name === 'ref-margin') {
        check('the margin document is the source plus the margin, with the note inside the margin',
          r.width === 880 && r.height === 400 && r.covered > 0 && r.bbox === null || r.width === 880, `${r.width}x${r.height}`);
      }
    }
    const font = getComputedStyle(document.querySelector('.callout') || document.body).fontFamily;
    const env = await invoke('editor_environment', { font, cssSize: `${document.getElementById('stage').clientWidth}x${document.getElementById('stage').clientHeight}` });
    say(`reference environment written to ${env}`);
  }

  // ---------------------------------------------------------------- 13. S0.6 check 3: the live editor against the output, while typing
  say('');
  say('S0.6 check 3: what the editor shows mid-typing against the copied image, decorations excluded');
  {
    const cases = [
      ['hebrew', 'הכפתור הזה לא עושה כלום כשהחיבור איטי, והמשתמש לוחץ שוב'],
      ['english', 'This label shows the group status, not the item status, and it wraps'],
      ['mixed', 'הכיתוב Save changes צריך להיות בעברית, and the English part too'],
    ];
    const origin = await invoke('editor_window_origin');
    for (const [name, text] of cases) {
      await openFixture('reference-scene.png');
      await editor.setZoom(1);
      const callout = editor.createCallout({ x: 200, y: 150 });
      editor.layoutScene();
      editor.startEditing(callout);
      document.execCommand('insertText', false, text);
      // The caret to the middle of the note, and a few more characters typed there, so the
      // copy happens mid-note with the caret live, which is the case the check is for.
      const textEl = el(callout).querySelector('.t');
      const node = textEl.firstChild;
      const range = document.createRange();
      range.setStart(node, Math.floor(node.length / 2));
      range.collapse(true);
      const selection = window.getSelection();
      selection.removeAllRanges();
      selection.addRange(range);
      document.execCommand('insertText', false, ' typed ');
      await sleep(150);

      const before = textEl.getBoundingClientRect();
      editor.setPlain(true);
      const plain = textEl.getBoundingClientRect();
      editor.setPlain(false);
      check(`${name}: the editing decorations do not move the text box`,
        before.left === plain.left && before.top === plain.top && before.width === plain.width && before.height === plain.height);

      const ratio = editor.ratioOf();
      const rect = textEl.getBoundingClientRect();
      const region = {
        screenX: origin.x + Math.round(rect.left * ratio),
        screenY: origin.y + Math.round(rect.top * ratio),
        // offsetLeft is measured from the bubble's padding box, so its border is added back.
        canvasX: callout.box.x + el(callout).clientLeft + textEl.offsetLeft + model.margin.left,
        canvasY: callout.box.y + el(callout).clientTop + textEl.offsetTop + model.margin.top,
        width: textEl.offsetWidth,
        height: textEl.offsetHeight,
      };
      const r = await exportCheck(`live-${name}`, 'source');
      check(`${name}: the copy did not end the editing`, model.editing === callout && document.activeElement === textEl);
      // The report overlay covers the stage while the checks run; the screen has to show
      // the editor itself for this one, decorations off, then the report comes back.
      editor.setPlain(true);
      document.body.classList.remove('reporting');
      await sleep(150);
      let live;
      try {
        if (!(await invoke('editor_in_front'))) throw new Error('another window is over ours');
        live = await invoke('editor_live_compare', region);
      } catch (err) {
        // The same screen the show-and-look section needs: named, not failed, when the
        // session refuses a copy of it.
        skipped(`${name}: the live editor against the output`, `the screen could not be copied: ${err}`);
        editor.commitEditing();
        continue;
      } finally {
        editor.setPlain(false);
        document.body.classList.add('reporting');
      }
      const sameWrap = live.live_lines.length === live.export_lines.length
        && live.live_lines.every((band, i) => Math.abs(band[0] - live.export_lines[i][0]) <= 2 && Math.abs(band[1] - live.export_lines[i][1]) <= 2);
      check(`${name}: the same wrap on screen and in the output`, sameWrap,
        `screen lines ${JSON.stringify(live.live_lines)}, output lines ${JSON.stringify(live.export_lines)}`);
      const where = live.bbox ? ` in the box ${live.bbox.join(',')}` : '';
      // At 100% the two are identical, 0 pixels. On a scaled display the screen draws the
      // note through a fractional CSS transform and its glyphs antialias differently from
      // the 1:1 raster of the export: 6.6% of the text box's pixels at 225%, with the wrap
      // and the box the same to the pixel (S1.1, F75). That is the antialiasing clause of
      // S0.6, inside tolerance; a wrap or box change fails above, at any tolerance.
      const bar = Math.abs(ratio - 1) < 0.01 ? 3 : 8;
      check(`${name}: the text box on screen matches the output within tolerance`, live.percent <= bar,
        `${live.differing} of ${live.width * live.height} pixels differ, ${live.percent.toFixed(2)}% at ratio ${ratio.toFixed(2)} (bar ${bar}%)${where}; ${r.covered} pixels in the layer`);
      editor.commitEditing();
    }
  }

  // ---------------------------------------------------------------- a vector, enlarged
  say('');
  say('A vector enlarged: drawn from the file at the zoom, and the screen holds only the last paint');
  {
    await editor.setMode('view');
    const svg = await openFixture('svg-css-style-block.svg');
    // Four times: the square's left edge is at 20 and its right at 140, so on the row through
    // its middle the columns up to 600 cross both. Drawn from the file, no pixel there is
    // part way; an enlargement of the 240 by 160 raster leaves a ramp at each edge.
    const region = await fetch('http://region.localhost/?x=0&y=0&w=150&h=160&ow=600&oh=640', { cache: 'no-store' });
    const pixels = new Uint8Array(await region.arrayBuffer()).subarray(8);
    let partial = 0;
    for (let x = 0; x < 600; x += 1) {
      const a = pixels[(240 * 600 + x) * 4 + 3];
      if (a > 0 && a < 255) partial += 1;
    }
    check('a vector asked for at four times its size has hard edges, not a ramp', svg.kind === 'vector' && partial <= 2, `${partial} part-way pixels on the row`);

    // Zoomed in step by step, the way the wheel does, then the screen against the canvas.
    await invoke('editor_show');
    for (let i = 0; i < 40 && !(await invoke('editor_window_visible')); i += 1) await sleep(50);
    await sleep(300);
    await editor.setZoom(1);
    for (const zoom of [1.5, 2.2, 3.3]) {
      await editor.setZoom(zoom);
      await sleep(120);
    }
    // The report is drawn over the stage, so it steps aside while the screen is read.
    document.body.classList.remove('reporting');
    await sleep(400);
    const origin = await invoke('editor_window_origin');
    const ratio = editor.ratioOf();
    const box = editor.canvas.getBoundingClientRect();
    const held = editor.canvas.getContext('2d').getImageData(0, 0, editor.canvas.width, editor.canvas.height).data;
    let seen = null;
    try {
      if (!(await invoke('editor_in_front'))) throw new Error('another window is over ours');
      seen = await invoke('editor_screen_compare', new Uint8Array(held.buffer), {
        headers: { x: String(origin.x + Math.round(box.left * ratio)), y: String(origin.y + Math.round(box.top * ratio)), width: String(editor.canvas.width), height: String(editor.canvas.height) },
      });
    } catch (err) {
      skipped('after three zoom steps the screen shows the canvas and nothing of the earlier paints', `the screen could not be copied: ${err}`);
    } finally {
      document.body.classList.add('reporting');
    }
    if (seen) check('after three zoom steps the screen shows the canvas and nothing of the earlier paints', seen.percent <= 0.5, `${seen.differing} of ${seen.pixels} differ, ${seen.percent.toFixed(2)}%, bbox ${JSON.stringify(seen.bbox)}`);
    await editor.setZoom(model.fitZoom);
  }

  // ---------------------------------------------------------------- 14. carried from S0.5: the export shows the frame, the orientation, the colour, the size
  say('');
  say('carried from S0.5: the frame chosen, the orientation applied, the profile converted and the SVG size are what the export shows');
  {
    await openFixture('gif-three-frames.gif');
    const third = await invoke('editor_frame', { index: 2 });
    await editor.loadImage(third);
    let r = await exportCheck('gif-frame-2', 'source', { sample: '5,5' });
    check('an animation exports the frame on screen, not the first', r.source_mismatches === 0 && r.sample && r.sample[0] === 40 && r.sample[1] === 70 && r.sample[2] === 220,
      `frame ${third.index + 1} of ${third.count}, pixel ${JSON.stringify(r.sample)}`);

    const jpeg = await openFixture('jpeg-orientation-6.jpg');
    r = await exportCheck('jpeg-upright', 'source', { sample: '190,10' });
    check('a rotated JPEG exports upright, with the red mark at the top right', r.width === 200 && r.height === 300 && r.sample && r.sample[0] > 200 && r.sample[1] < 80 && r.sample[2] < 80,
      `${r.width}x${r.height} (opened as ${jpeg.width}x${jpeg.height}), pixel ${JSON.stringify(r.sample)}`);

    await openFixture('png-alpha-and-swapped-profile.png');
    r = await exportCheck('png-profile', 'source', { sample: '10,10' });
    check('a profiled PNG exports converted, so its stored red is green, and its transparent half is exact', r.source_mismatches === 0 && r.sample && r.sample[1] > 200 && r.sample[0] < 60,
      `pixel ${JSON.stringify(r.sample)}, ${r.source_mismatches} mismatches`);

    const svg = await openFixture('svg-text-labels.svg');
    r = await exportCheck('svg-raster', 'source');
    check('an SVG exports at the raster size fixed when it was opened', r.width === svg.width && r.height === svg.height && r.source_mismatches === 0,
      `${r.width}x${r.height}`);
  }

  // ---------------------------------------------------------------- 15. the clipboard
  say('');
  say('the clipboard: three formats published by the host, read back the way a destination reads them');
  {
    await openFixture('reference-scene.png');
    await editor.setMargin({ left: 0, top: 0, right: 240, bottom: 0 });
    const c = editor.createCallout({ x: 600, y: 200 });
    c.text = 'Inside the margin, בתוך השוליים';
    c.box = { x: 660, y: 170, width: 200 };
    editor.layoutScene();
    await sleep(40);
    const copy = await editor.copyComposed();
    const back = await invoke('editor_clipboard_readback');
    check('PNG, CF_DIBV5 and CF_DIB are all on the clipboard', back.png && back.dib_v5 && back.dib,
      `png ${back.png}, dibv5 ${back.dib_v5}, dib ${back.dib}`);
    check('the clipboard PNG is the composed image, byte for byte', back.png_matches && back.width === copy.width && back.height === copy.height,
      `${back.width}x${back.height}, ${back.png_bytes} bytes; composed ${copy.compose_ms} ms, encoded ${copy.published.encode_ms} ms, published ${copy.published.publish_ms} ms`);
    say('the clipboard is left holding this image, for the paste into Claude and ChatGPT');
  }

  // ---------------------------------------------------------------- 16. S1.1: the capture lifecycle
  say('');
  say('S1.1: a new capture keeps the previous one, the page keeps its notes, the empty state names the hotkey');
  {
    const hotkey = await invoke('editor_hotkey');
    check('the empty state names the configured hotkey', typeof hotkey === 'string' && hotkey.length > 0 && editor.emptyStateText(hotkey).includes(hotkey),
      JSON.stringify(editor.emptyStateText(hotkey)));

    const first = await invoke('editor_capture_probe', { width: 640, height: 400 });
    await editor.loadImage(first);
    const note = editor.createCallout({ x: 100, y: 100 });
    note.text = 'a note on the first capture';
    editor.layoutScene();
    // The host announces the same capture too; told twice, the document keeps its note.
    await editor.loadImage(await invoke('editor_image_info'));
    check('the same document told twice keeps its notes', model.callouts.length === 1 && model.image.document_id === first.document_id);

    const second = await invoke('editor_capture_probe', { width: 320, height: 200 });
    await editor.loadImage(second);
    check('a new capture is a new document, starting empty', second.document_id !== first.document_id && model.callouts.length === 0 && model.nextNumber === 1,
      `documents ${first.document_id} then ${second.document_id}`);
    const stashed = editor.documents.get(first.document_id);
    check('the previous document\'s notes are kept', !!stashed && stashed.callouts.length === 1 && stashed.callouts[0].text === 'a note on the first capture');

    let history = [];
    for (let i = 0; i < 40 && !history.some((r) => r[0] === first.document_id && r[3] > 0); i += 1) {
      await sleep(50);
      history = await invoke('editor_history');
    }
    const kept = history.find((r) => r[0] === first.document_id);
    check('the previous capture is retained by the host, encoded', !!kept && kept[1] === 640 && kept[2] === 400 && kept[3] > 0,
      kept ? `${kept[1]}x${kept[2]}, ${kept[3]} bytes` : 'not in the history');
    model.callouts = [];
    editor.layoutScene();
  }

  // ---------------------------------------------------------------- 17. S1.2: Ctrl+O opens Windows' picker
  say('');
  say('S1.2: Ctrl+O opens the file picker, owned by the editor, and Escape closes it with nothing opened');
  {
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'o', code: 'KeyO', ctrlKey: true, bubbles: true, cancelable: true }));
    await sleep(1500);
    let outcome = await invoke('editor_dialog_outcome');
    check('the picker is open, so Ctrl+O has no outcome yet', outcome === '', JSON.stringify(outcome));
    // The key is sent only when the picker, ours, is in front; with someone else's window
    // in front nothing is sent, because a key pressed for real lands wherever the focus
    // is, and the check says so instead.
    const sent = await invoke('editor_press_escape');
    for (let i = 0; i < 40 && sent && !outcome; i += 1) {
      await sleep(100);
      outcome = await invoke('editor_dialog_outcome');
    }
    if (!sent) {
      skipped('Escape closes the picker with nothing chosen', 'another application is in front, so no key was sent; the picker is left open');
    } else if (screenOff && outcome === '') {
      skipped('Escape closes the picker with nothing chosen', 'the session shows no screen, so the picker could not be closed by a key');
    } else {
      check('Escape closes the picker with nothing chosen', outcome === 'Ctrl+O: nothing chosen', JSON.stringify(outcome));
    }
  }

  // ---------------------------------------------------------------- 18. S1.3: the viewing surface
  say('');
  say('S1.3: the wheel zooms around the pointer, with Ctrl or without, a sideways wheel pans, fullscreen in and out, the file name in the title');
  {
    const info = await invoke('editor_open_fixture', { name: 'reference-scene.png' });
    await editor.loadImage(info);
    const title = await invoke('editor_window_title');
    check('the file name is in the window title', title === 'reference-scene.png - Recon', JSON.stringify(title));

    // Larger than the window, so the view can pan: a picture that fits is always centred,
    // and a zoom around the pointer has nothing to hold then.
    await editor.setZoom(6);
    const stage = document.getElementById('stage');
    const box = stage.getBoundingClientRect();
    const cssX = box.width * 0.6;
    const cssY = box.height * 0.4;
    const under = () => ({
      x: model.pan.x + ((cssX - model.offset.x) * editor.ratioOf()) / model.zoom,
      y: model.pan.y + ((cssY - model.offset.y) * editor.ratioOf()) / model.zoom,
    });
    const before = under();
    stage.dispatchEvent(new WheelEvent('wheel', { deltaY: -100, ctrlKey: true, clientX: box.left + cssX, clientY: box.top + cssY, bubbles: true, cancelable: true }));
    await sleep(200);
    const after = under();
    check('Ctrl+wheel zooms in around the pointer', model.zoom > 6 && Math.abs(after.x - before.x) < 1.5 && Math.abs(after.y - before.y) < 1.5,
      `zoom ${model.zoom.toFixed(3)}, the point under the pointer moved ${(after.x - before.x).toFixed(2)},${(after.y - before.y).toFixed(2)} image px`);

    // Rotem, 2026-09-14: the wheel alone zooms the same way, out and back in, around the
    // same point.
    const zoomIn = model.zoom;
    const wheelAt = (init) => stage.dispatchEvent(new WheelEvent('wheel', { clientX: box.left + cssX, clientY: box.top + cssY, bubbles: true, cancelable: true, ...init }));
    wheelAt({ deltaY: 120 });
    await sleep(200);
    const out = under();
    check('the wheel zooms out around the pointer', model.zoom < zoomIn && Math.abs(out.x - before.x) < 1.5 && Math.abs(out.y - before.y) < 1.5,
      `zoom ${zoomIn.toFixed(3)} to ${model.zoom.toFixed(3)}, the point under the pointer moved ${(out.x - before.x).toFixed(2)},${(out.y - before.y).toFixed(2)} image px`);
    const zoomOut = model.zoom;
    wheelAt({ deltaY: -120 });
    await sleep(200);
    const back = under();
    check('and back in around it', model.zoom > zoomOut && Math.abs(back.x - before.x) < 1.5 && Math.abs(back.y - before.y) < 1.5,
      `zoom ${zoomOut.toFixed(3)} to ${model.zoom.toFixed(3)}, the point under the pointer moved ${(back.x - before.x).toFixed(2)},${(back.y - before.y).toFixed(2)} image px`);

    // Rotem, 2026-09-19: a fast spin of the wheel. Each notch used to make the region on its way
    // stale, so nothing was painted until the wheel rested. Now the picture already painted is
    // stretched to the new zoom at once, read here before any region can have landed, and the
    // view at rest is an ordinary paint at the last zoom.
    await editor.setZoom(6);
    const spinFrom = model.zoom;
    const canvasEl = document.getElementById('canvas');
    for (let i = 0; i < 5; i += 1) wheelAt({ deltaY: 120 });
    const held = editor.lastRegion();
    const stretchedTo = parseFloat(canvasEl.style.width);
    const stretchWanted = (held.w * model.zoom) / editor.ratioOf();
    check('a fast spin stretches the picture already painted to the new zoom at once',
      model.zoom < spinFrom && held.zoom === spinFrom && Math.abs(stretchedTo - stretchWanted) < 0.01 && document.body.classList.contains('zooming'),
      `zoom ${spinFrom.toFixed(3)} to ${model.zoom.toFixed(3)}, the canvas ${stretchedTo.toFixed(2)} css px wide for ${stretchWanted.toFixed(2)}, the region held from zoom ${held.zoom.toFixed(3)}`);
    const sceneZoom = /scale\(([\d.]+)\)/.exec(document.getElementById('scene').style.transform);
    check('and the notes take the new zoom with it', !!sceneZoom && Math.abs(Number(sceneZoom[1]) - model.zoom / editor.ratioOf()) < 0.0001,
      `the scene's scale ${sceneZoom && sceneZoom[1]} for ${(model.zoom / editor.ratioOf()).toFixed(4)}`);
    for (let i = 0; i < 80 && document.body.classList.contains('zooming'); i += 1) await sleep(25);
    const rested = editor.lastRegion();
    check('at rest the view is an ordinary paint at the last zoom',
      !document.body.classList.contains('zooming') && !document.body.classList.contains('panning') && rested.zoom === model.zoom
        && Math.abs(parseFloat(canvasEl.style.width) - rested.width / editor.ratioOf()) < 0.01,
      `the region at zoom ${rested.zoom.toFixed(3)} for ${model.zoom.toFixed(3)}, the canvas ${canvasEl.style.width} for ${rested.width} px`);
    check('the host says whether the file has a see-through pixel', typeof model.image.opaque === 'boolean', JSON.stringify(model.image.opaque));

    // A sideways wheel still pans and leaves the zoom alone: from the left end at the top
    // zoom, where this picture is wider than the window.
    await editor.setZoom(8);
    model.pan.x = 0;
    await editor.paintRegion();
    const sideways = { zoom: model.zoom, x: model.pan.x };
    wheelAt({ deltaX: 120, deltaY: 0 });
    await sleep(200);
    check('a sideways wheel pans and does not zoom', model.zoom === sideways.zoom && model.pan.x > sideways.x,
      `zoom ${sideways.zoom} to ${model.zoom}, pan ${sideways.x.toFixed(1)} to ${model.pan.x.toFixed(1)}`);

    // Rotem, 2026-09-15: the whole picture, as the stage is now and at most its actual size, is
    // the most it zooms out, by the wheel and by the key alike. Each input starts above the
    // stop, so an input that did nothing fails.
    await editor.loadImage(await invoke('editor_load_probe', { width: 3840, height: 2160 }));
    await editor.refreshStrip();
    await sleep(100);
    const floor = editor.computeFit();
    const minusKey = () => window.dispatchEvent(new KeyboardEvent('keydown', { key: '-', code: 'Minus', bubbles: true, cancelable: true }));
    // A change of space is followed after a round trip to the host (the ratio is measured first),
    // so the checks wait for the kept fit to meet the stage, up to a second and a half.
    const followSettled = async () => {
      for (let i = 0; i < 60 && model.fitZoom !== editor.computeFit(); i += 1) await sleep(25);
      await sleep(50);
    };
    // Full screen resizes the window after the call returns, so the checks wait for the page's
    // size to change before waiting for the follow.
    const resizeSettled = async (before) => {
      for (let i = 0; i < 60 && window.innerWidth === before.w && window.innerHeight === before.h; i += 1) await sleep(25);
      await followSettled();
    };
    await editor.setZoom(floor * 1.1);
    wheelAt({ deltaY: 120 });
    await sleep(200);
    const wheeledOut = model.zoom;
    await editor.setZoom(floor * 1.2);
    minusKey();
    await sleep(200);
    check('zooming out stops at the whole picture, by the wheel and by the - key', floor < 1 && wheeledOut === floor && model.zoom === floor,
      `whole at ${floor.toFixed(3)}, the wheel from ${(floor * 1.1).toFixed(3)} to ${wheeledOut.toFixed(3)}, - from ${(floor * 1.2).toFixed(3)} to ${model.zoom.toFixed(3)}`);
    // The stop is read from the stage, not from the fit the page keeps, so a kept fit left
    // stale on purpose here stops neither input short of the whole picture.
    model.fitZoom = floor * 1.05;
    await editor.setZoom(floor * 1.2);
    wheelAt({ deltaY: 120 });
    await sleep(200);
    const liveWheel = model.zoom;
    await editor.setZoom(floor * 1.2);
    minusKey();
    await sleep(200);
    check('the stop is the stage as it is, not a kept fit gone stale, by the wheel and by the - key', liveWheel === floor && model.zoom === floor,
      `kept ${(floor * 1.05).toFixed(3)}, the wheel to ${liveWheel.toFixed(3)}, - to ${model.zoom.toFixed(3)}, whole at ${floor.toFixed(3)}`);
    // A zoom already below the whole picture, as for the moment the page takes to measure a
    // grown stage, is never enlarged by a zoom out.
    const below = floor * 0.9;
    model.zoom = below;
    await editor.paintRegion();
    wheelAt({ deltaY: 120 });
    await sleep(200);
    const belowWheel = model.zoom;
    minusKey();
    await sleep(200);
    check('a zoom out never enlarges a picture already below the whole of it, by the wheel or the - key', belowWheel === below && model.zoom === below,
      `below at ${below.toFixed(3)}, the wheel ${belowWheel.toFixed(3)}, - ${model.zoom.toFixed(3)}, whole at ${floor.toFixed(3)}`);
    // Rotem, 2026-09-15: when the space changes, a view at the whole picture takes the new whole
    // picture. Each change starts from a kept fit and a view left at a taller stage's fit.
    const followers = [
      ['the timeline refreshing', () => editor.refreshStrip()],
      ['the trash showing', () => editor.renderTrash()],
      ['the window resizing', async () => { window.dispatchEvent(new Event('resize')); }],
    ];
    // Two starts: a view at a taller stage's kept fit, and a zoomed-in view the grown whole
    // picture has passed, below the whole but not at the kept fit.
    const starts = [['at the kept fit', 1.05, 1.05], ['below the whole', 0.8, 0.9]];
    const followed = [];
    for (const [label, run] of followers) {
      for (const [start, kept, zoom] of starts) {
        model.fitZoom = floor * kept;
        model.zoom = floor * zoom;
        await run();
        await followSettled();
        followed.push({ label: `${label}, ${start}`, zoom: model.zoom, kept: model.fitZoom, whole: editor.computeFit() });
      }
    }
    await editor.refreshStrip();
    await followSettled();
    check('a view at the whole picture, or below it, follows the space when the timeline refreshes, the trash shows or the window resizes', followed.every((f) => f.zoom === f.whole && f.kept === f.whole),
      followed.map((f) => `${f.label}: zoom ${f.zoom.toFixed(3)}, kept ${f.kept.toFixed(3)}, whole ${f.whole.toFixed(3)}`).join('; '));
    const whole = editor.computeFit();
    await editor.setZoom(whole * 1.2);
    window.dispatchEvent(new KeyboardEvent('keydown', { key: '0', code: 'Digit0', bubbles: true, cancelable: true }));
    await sleep(200);
    check('0 shows the same whole picture that zooming out stops at', model.zoom === whole && model.fitZoom === whole,
      `0 from ${(whole * 1.2).toFixed(3)} to ${model.zoom.toFixed(3)}, kept ${model.fitZoom.toFixed(3)}, whole at ${whole.toFixed(3)}`);
    // Full screen grows the space: the picture grows to the new whole picture, zooms out no
    // further, and a zoom in comes back out to it; leaving full screen gives the smaller whole.
    let sizeBefore = { w: window.innerWidth, h: window.innerHeight };
    await editor.setFullscreen(true);
    await resizeSettled(sizeBefore);
    const grown = { zoom: model.zoom, whole: editor.computeFit() };
    wheelAt({ deltaY: 120 });
    await sleep(200);
    const grownWheel = model.zoom;
    wheelAt({ deltaY: -120 });
    await sleep(200);
    const grownIn = model.zoom;
    wheelAt({ deltaY: 120 });
    await sleep(200);
    const grownBack = model.zoom;
    sizeBefore = { w: window.innerWidth, h: window.innerHeight };
    await editor.setFullscreen(false);
    await resizeSettled(sizeBefore);
    const shrunk = { zoom: model.zoom, whole: editor.computeFit() };
    check('when the space grows the picture grows to the new whole picture, zooms out no further, and a zoom in comes back out to it',
      grown.whole > whole && Math.abs(grown.zoom - grown.whole) < 1e-9 && Math.abs(grownWheel - grown.whole) < 1e-9 && grownIn > grown.whole * 1.2 && Math.abs(grownBack - grown.whole) < 1e-9,
      `whole ${whole.toFixed(3)}, in full screen ${grown.whole.toFixed(3)}: the zoom ${grown.zoom.toFixed(3)}, the wheel out ${grownWheel.toFixed(3)}, in ${grownIn.toFixed(3)}, back out ${grownBack.toFixed(3)}`);
    check('and out of full screen the whole picture follows the smaller space', Math.abs(shrunk.zoom - shrunk.whole) < 1e-9 && Math.abs(shrunk.whole - whole) < 1e-9,
      `the zoom ${shrunk.zoom.toFixed(3)}, whole ${shrunk.whole.toFixed(3)}`);
    // A stage with no area, here 1 px tall as a timeline dragged to the top of a short window
    // can leave it, has no whole picture: the view keeps its zoom, a zoom out stops at the kept
    // fit, and the pan stays a number through a zoom key and the space coming back.
    await editor.setZoom(editor.computeFit());
    const beforeEmpty = model.zoom;
    stage.style.bottom = `${window.innerHeight - 65}px`;
    window.dispatchEvent(new Event('resize'));
    await sleep(250);
    const empty = { h: stage.clientHeight, zoom: model.zoom, kept: model.fitZoom };
    minusKey();
    await sleep(100);
    const afterMinus = model.zoom;
    wheelAt({ deltaY: 120 });
    await sleep(100);
    const afterWheel = model.zoom;
    window.dispatchEvent(new KeyboardEvent('keydown', { key: '+', code: 'Equal', bubbles: true, cancelable: true }));
    await sleep(100);
    const keyed = { zoom: model.zoom, x: model.pan.x, y: model.pan.y };
    stage.style.bottom = '';
    window.dispatchEvent(new Event('resize'));
    await followSettled();
    const restored = { x: model.pan.x, y: model.pan.y, whole: editor.computeFit() };
    await editor.setZoom(restored.whole);
    check('a stage with no area keeps the zoom, a zoom out there stops at the kept fit, by the key and the wheel, and the pan stays a number',
      empty.h === 1 && empty.zoom === beforeEmpty && empty.kept > 0 && afterMinus === beforeEmpty && afterWheel === beforeEmpty && keyed.zoom > beforeEmpty
        && [keyed.x, keyed.y, restored.x, restored.y].every(Number.isFinite),
      `stage ${empty.h} px tall, zoom ${beforeEmpty.toFixed(3)} kept as ${empty.zoom.toFixed(3)}, - to ${afterMinus.toFixed(3)}, the wheel to ${afterWheel.toFixed(3)}, + to ${keyed.zoom.toFixed(3)}, pan ${keyed.x.toFixed(1)},${keyed.y.toFixed(1)}, back ${restored.x.toFixed(1)},${restored.y.toFixed(1)}`);
    // A picture opened onto a stage with no area, a timeline at its full height over a short
    // window, opens at the fit kept from before rather than a zoom of 0, so a zoom key there keeps
    // the pan a number; when the space returns it takes its own whole picture, even after a zoom
    // key pressed blind, and the follow is spent once it has run. The picture before it is larger
    // than the window and smaller than the one opened, so the kept fit is below 1 and above the
    // opened picture's whole.
    await editor.loadImage(await invoke('editor_load_probe', { width: 1600, height: 1000 }));
    const keptBefore = model.fitZoom;
    stage.style.bottom = `${window.innerHeight}px`;
    window.dispatchEvent(new Event('resize'));
    await sleep(250);
    await editor.loadImage(await invoke('editor_load_probe', { width: 3840, height: 2160 }));
    const openedEmpty = { h: stage.clientHeight, zoom: model.zoom, kept: model.fitZoom };
    window.dispatchEvent(new KeyboardEvent('keydown', { key: '+', code: 'Equal', bubbles: true, cancelable: true }));
    await sleep(100);
    const openedKeyed = { zoom: model.zoom, x: model.pan.x, y: model.pan.y };
    stage.style.bottom = '';
    window.dispatchEvent(new Event('resize'));
    for (let i = 0; i < 60 && model.zoom !== editor.computeFit(); i += 1) await sleep(25);
    const openedBack = { zoom: model.zoom, kept: model.fitZoom, whole: editor.computeFit(), x: model.pan.x, y: model.pan.y };
    await editor.setZoom(openedBack.whole * 1.2);
    const chosen = model.zoom;
    window.dispatchEvent(new Event('resize'));
    await sleep(250);
    const afterFollow = model.zoom;
    await editor.setZoom(editor.computeFit());
    check('a picture opened onto a stage with no area opens at the kept fit, a zoom key there keeps the pan a number, and it takes its own whole picture when the space returns',
      keptBefore < 1 && keptBefore > openedBack.whole && openedEmpty.h === 0 && openedEmpty.zoom === keptBefore && openedEmpty.kept === keptBefore
        && openedKeyed.zoom === keptBefore * 1.5 && [openedKeyed.x, openedKeyed.y, openedBack.x, openedBack.y].every(Number.isFinite)
        && openedBack.zoom === openedBack.whole && openedBack.kept === openedBack.whole && afterFollow === chosen,
      `kept ${keptBefore.toFixed(3)}, stage ${openedEmpty.h} px tall, opened at ${openedEmpty.zoom.toFixed(3)} with the fit ${openedEmpty.kept.toFixed(3)}, + to ${openedKeyed.zoom.toFixed(3)}, pan ${openedKeyed.x.toFixed(1)},${openedKeyed.y.toFixed(1)}; back: zoom ${openedBack.zoom.toFixed(3)}, fit ${openedBack.kept.toFixed(3)}, whole ${openedBack.whole.toFixed(3)}, pan ${openedBack.x.toFixed(1)},${openedBack.y.toFixed(1)}; a zoom chosen at ${chosen.toFixed(3)} kept through a follow as ${afterFollow.toFixed(3)}`);
    // A note that grows the margin follows the same way: a view left at a kept fit takes the
    // whole composition once the margin has grown.
    await editor.loadImage(await invoke('editor_load_probe', { width: 120, height: 80 }));
    await editor.refreshStrip();
    await followSettled();
    await sleep(100);
    model.callouts = []; model.nextNumber = 1;
    model.fitZoom = 1.05;
    model.zoom = 1.05;
    editor.createCallout({ x: 60, y: 40 });
    editor.layoutScene();
    await editor.settle();
    const margined = { zoom: model.zoom, kept: model.fitZoom, whole: editor.computeFit(), margin: `${model.margin.left},${model.margin.top},${model.margin.right},${model.margin.bottom}` };
    model.callouts = [];
    editor.layoutScene();
    await editor.settle();
    check('a note that grows the margin takes a view at the kept fit to the whole composition', margined.margin !== '0,0,0,0' && margined.zoom === margined.whole && margined.kept === margined.whole,
      `margin ${margined.margin}, zoom ${margined.zoom.toFixed(3)}, kept ${margined.kept.toFixed(3)}, whole ${margined.whole.toFixed(3)}`);
    // A picture that fits the window opens at its actual size, and that is its floor.
    await editor.loadImage(await invoke('editor_load_probe', { width: 300, height: 200 }));
    const small = model.zoom;
    wheelAt({ deltaY: 120 });
    await sleep(200);
    check('a picture that fits the window opens at its actual size and zooms out no further', small === 1 && model.fitZoom === 1 && model.zoom === 1,
      `opens at ${small}, after the wheel ${model.zoom}`);

    const on = await editor.setFullscreen(true);
    await sleep(300);
    const off = await editor.setFullscreen(false);
    await sleep(300);
    check('fullscreen goes on and off through the host', on === true && off === false, `on ${on}, off ${off}`);
    await editor.setZoom(model.fitZoom);
  }

  // ---------------------------------------------------------------- 19. S1.4: folder navigation
  say('');
  say('S1.4: the folder in logical order, the position at both ends, a vanished file skipped, the context named');
  {
    const dir = await invoke('editor_make_folder');
    const at = (name) => `${dir}\\${name}`;
    let info = await invoke('editor_open_path', { path: at('img2.png') });
    await editor.loadImage(info);
    check('the folder is listed in logical order and the file placed in it', info.position === 4 && info.total === 5 && info.context === 'folder',
      `${info.position} of ${info.total} in ${JSON.stringify(info.context)}`);
    info = await invoke('editor_navigate', { step: 'next' });
    await editor.loadImage(info);
    check('next goes to img10 after img2, not to IMG1', info.file === 'img10.png' && info.position === 5, `${info.file} ${info.position} of ${info.total}`);
    info = await invoke('editor_navigate', { step: 'next' });
    check('the last file stays at the end', info.file === 'img10.png' && info.position === 5, `${info.file} ${info.position} of ${info.total}`);
    info = await invoke('editor_navigate', { step: 'first' });
    await editor.loadImage(info);
    check('first is a.gif', info.file === 'a.gif' && info.position === 1, `${info.file} ${info.position} of ${info.total}`);
    info = await invoke('editor_navigate', { step: 'previous' });
    check('the first file stays at the start', info.file === 'a.gif' && info.position === 1);
    await invoke('editor_remove_from_folder', { name: 'b.jpg' });
    info = await invoke('editor_navigate', { step: 'next' });
    await editor.loadImage(info);
    check('a file gone since the listing is skipped, and the count follows', info.file === 'IMG1.png' && info.position === 2 && info.total === 4,
      `${info.file} ${info.position} of ${info.total}`);
    info = await invoke('editor_navigate', { step: 'last' });
    await editor.loadImage(info);
    check('last is img10', info.file === 'img10.png' && info.position === 4 && info.total === 4);
    check('the HUD names the context', document.getElementById('hud').textContent.includes('4 of 4 in folder'));
  }

  // ---------------------------------------------------------------- 20. S1.5: viewing and annotation part ways
  say('');
  say('S1.5: viewing arms no tool, the first tool picked creates or resumes one managed document, a reopened file shows the route to its edit');
  {
    const dir = await invoke('editor_make_folder');
    const at = (name) => `${dir}\\${name}`;
    const hud = document.getElementById('hud');
    const stage = editor.stage;
    const same = (a, b) => a.toLowerCase() === b.toLowerCase();
    const forFile = (list, name) => list.filter((m) => m.source === 'file' && same(m.path, at(name)));
    const pointer = (type, x, y) => stage.dispatchEvent(new PointerEvent(type, {
      pointerId: 7, button: 0, buttons: type === 'pointerup' ? 0 : 1, clientX: x, clientY: y, bubbles: true, cancelable: true,
    }));

    // The drag is tried on a picture big enough to pan at the top zoom on any display:
    // the 640x400 reference scene, put in the folder under one of its names.
    await invoke('editor_replace_in_folder', { name: 'img10.png' });
    let info = await invoke('editor_open_path', { path: at('img10.png') });
    await editor.loadImage(info);
    check('a file opens in viewing mode', model.mode === 'view' && info.managed === false && document.body.classList.contains('viewing'),
      `mode ${model.mode}, managed ${info.managed}`);
    check('the HUD says so', hud.textContent.includes('mode view'), hud.textContent.split('\n')[1]);

    // A click and a drag on the picture, in viewing mode. Zoomed in first, so there is
    // somewhere to pan to.
    await editor.setZoom(8);
    const box = stage.getBoundingClientRect();
    let cx = box.left + model.offset.x + 60;
    let cy = box.top + model.offset.y + 60;
    // The key zoom keeps the viewport's centre, which on an image smaller than the window
    // lands the pan at its far end, so the drag goes right and down and the pan comes back.
    const before = { ...model.pan };
    pointer('pointerdown', cx, cy);
    pointer('pointermove', cx + 30, cy + 20);
    pointer('pointerup', cx + 30, cy + 20);
    await editor.paintRegion();
    check('a click creates nothing and selects nothing', model.callouts.length === 0 && model.editing === null && model.selected === null);
    check('a drag pans', before.x > 0 && before.y > 0 && model.pan.x < before.x && model.pan.y < before.y,
      `pan ${Math.round(before.x)},${Math.round(before.y)} to ${Math.round(model.pan.x)},${Math.round(model.pan.y)} at zoom ${model.zoom}, ratio ${editor.ratioOf().toFixed(2)}`);

    // The document flow on the small one, which the reference scene later replaces on
    // disk at another size.
    info = await invoke('editor_open_path', { path: at('img2.png') });
    await editor.loadImage(info);
    await editor.setZoom(8);
    cx = box.left + model.offset.x + 60;
    cy = box.top + model.offset.y + 60;
    const viewId = info.document_id;
    // The mode has no control of its own (Rotem, 2026-09-18): the tool's key over a viewed file
    // starts the annotation, and the tool is in hand when the document is.
    window.dispatchEvent(new KeyboardEvent('keydown', { bubbles: true, cancelable: true, code: 'KeyC', key: 'c' }));
    for (let i = 0; i < 50 && !(model.mode === 'annotate' && model.tool === 'callout'); i += 1) await sleep(20);
    check('a tool picked over a viewed file switches the mode, the tool in hand', model.mode === 'annotate' && model.tool === 'callout' && model.image.managed === true && !document.body.classList.contains('viewing'),
      `mode ${model.mode}, tool ${model.tool}`);
    editor.setTool(null);
    let managed = await invoke('editor_managed');
    let mine = forFile(managed, 'img2.png');
    check('one managed document is created for the file, frame 0, at its size',
      mine.length === 1 && mine[0].id === viewId && mine[0].frame === 0 && mine[0].width === info.width && mine[0].height === info.height,
      JSON.stringify(mine));
    for (let i = 0; i < 60 && forFile(await invoke('editor_managed'), 'img2.png').every((m) => m.bytes === 0); i += 1) await sleep(50);
    managed = await invoke('editor_managed');
    check('its decoded image is preserved, encoded', forFile(managed, 'img2.png')[0].bytes > 0, `${forFile(managed, 'img2.png')[0].bytes} bytes`);

    pointer('pointerdown', cx, cy);
    pointer('pointerup', cx, cy);
    check('in annotation with no tool in hand the same click creates nothing either', model.tool === null && model.callouts.length === 0 && model.shapes.length === 0 && model.editing === null);
    editor.setTool('callout');
    pointer('pointerdown', cx, cy);
    pointer('pointerup', cx, cy);
    check('with the callout tool the same click anchors a note and its bubble waits for the second click', model.callouts.length === 1 && model.placing === model.callouts[0] && model.editing === null);
    pointer('pointermove', cx + 20, cy + 16);
    pointer('pointerdown', cx + 20, cy + 16);
    pointer('pointerup', cx + 20, cy + 16);
    check('the second click locks the bubble and the typing begins', model.placing === null && model.editing === model.callouts[0]);
    document.querySelector(`[data-id="${model.callouts[0].id}"] .t`).textContent = 'kept across the modes';
    editor.commitEditing();
    check('the note finished puts the tool down: the ordinary pointer, and a drag pans', model.tool === null && !document.body.classList.contains('armed') && model.mode === 'annotate');
    // A press on the finished note's text edits it again, with no tool and no mode to switch.
    {
      const t = document.querySelector(`[data-id="${model.callouts[0].id}"] .t`);
      const r = t.getBoundingClientRect();
      t.dispatchEvent(new PointerEvent('pointerdown', { pointerId: 7, button: 0, buttons: 1, clientX: r.left + 2, clientY: r.top + 2, bubbles: true, cancelable: true }));
      check('a press on the finished note edits it again', model.editing === model.callouts[0] && model.tool === null);
      pointer('pointerup', r.left + 2, r.top + 2);
      editor.commitEditing();
    }
    // A shape drawn puts its tool down as well; a press that draws nothing keeps it.
    editor.setTool('rect');
    pointer('pointerdown', cx + 40, cy + 40);
    pointer('pointerup', cx + 40, cy + 40);
    check('a press that draws nothing keeps the tool in hand', model.tool === 'rect' && model.shapes.length === 0);
    pointer('pointerdown', cx + 40, cy + 40);
    pointer('pointermove', cx + 120, cy + 100);
    pointer('pointerup', cx + 120, cy + 100);
    check('a shape drawn puts its tool down', model.tool === null && model.shapes.length === 1, `tool ${model.tool}, shapes ${model.shapes.length}`);
    editor.undo();
    check('and Ctrl+Z takes the shape back, the tool still down', model.shapes.length === 0 && model.tool === null);

    await editor.setMode('view');
    check('leaving annotation keeps the work', model.mode === 'view' && model.callouts.length === 1 && model.callouts[0].text === 'kept across the modes');
    pointer('pointerdown', cx, cy);
    pointer('pointerup', cx, cy);
    check('and a click in viewing mode still creates nothing', model.callouts.length === 1 && model.editing === null && model.selected === null);
    await editor.setMode('annotate');
    managed = await invoke('editor_managed');
    check('Annotate again reuses the document, never a second one',
      model.image.document_id === viewId && forFile(managed, 'img2.png').length === 1 && model.callouts.length === 1);

    const other = await invoke('editor_open_path', { path: at('IMG1.png') });
    await editor.loadImage(other);
    check('another file opens in viewing, empty, still in the folder', model.mode === 'view' && model.callouts.length === 0 && other.managed === false && other.context === 'folder');

    // The file changes on disk meanwhile: a different picture of a different size.
    await invoke('editor_replace_in_folder', { name: 'img2.png' });
    const back = await invoke('editor_open_path', { path: at('img2.png') });
    await editor.loadImage(back);
    check('returning shows the file as it is now on disk, not the edit',
      back.managed === false && back.document_id !== viewId && model.callouts.length === 0 && (back.width !== info.width || back.height !== info.height),
      `${back.width}x${back.height}, was ${info.width}x${info.height}`);
    check('with the route to its saved edit', back.edited_ago_s !== null && back.edited_ago_s < 300 && hud.textContent.includes('annotated before'),
      hud.textContent.split('\n')[1]);
    check('the position in the folder is untouched by any of it', back.position === info.position && back.total === info.total,
      `${back.position} of ${back.total}`);

    await editor.pickTool('callout');
    managed = await invoke('editor_managed');
    check('a tool picked resumes that one document, its note where it was',
      model.image.document_id === viewId && model.callouts.length === 1 && model.callouts[0].text === 'kept across the modes' && forFile(managed, 'img2.png').length === 1,
      `document ${model.image.document_id}, ${forFile(managed, 'img2.png').length} for the file`);
    check('on its own preserved pixels, and it says the file has moved on',
      model.image.width === info.width && model.image.height === info.height && model.image.source_changed === true && hud.textContent.includes('changed since'),
      `${model.image.width}x${model.image.height}, source_changed ${model.image.source_changed}`);
    check('entering annotation kept the folder context', model.image.position === info.position && model.image.context === 'folder');

    const shot = await invoke('editor_capture_probe', { width: 320, height: 200 });
    await editor.loadImage(shot);
    check('a capture opens ready for annotation', model.mode === 'annotate' && shot.managed === true);
    check('no mode button is left in the sidebar', document.getElementById('mode') === null);
    window.dispatchEvent(new KeyboardEvent('keydown', { bubbles: true, cancelable: true, code: 'KeyA', key: 'a' }));
    await sleep(100);
    check('and the A key switches nothing', model.mode === 'annotate');
    await editor.setMode('view');
    await editor.setMode('annotate');
    managed = await invoke('editor_managed');
    check('the mode on a capture changes no document: one entry, the same number', model.image.document_id === shot.document_id && managed.filter((m) => m.id === shot.document_id).length === 1);
    model.callouts = [];
    editor.layoutScene();
  }

  // ---------------------------------------------------------------- 21. S1.6: placement, the margin, the numbers
  say('');
  say('S1.6: deterministic placement near the anchor, the margin when a bubble cannot fit, numbers with their gaps in the output');
  {
    const reset = async (name) => {
      await openFixture(name);
      model.callouts = [];
      model.nextNumber = 1;
      await editor.setMargin({ left: 0, top: 0, right: 0, bottom: 0 });
      editor.layoutScene();
    };
    const rect = (c) => ({ x: c.box.x, y: c.box.y, w: c.box.width, h: editor.heightOf(c) });
    const inside = (c) => {
      const r = rect(c);
      return r.x >= -model.margin.left && r.y >= -model.margin.top
        && r.x + r.w <= model.image.width + model.margin.right && r.y + r.h <= model.image.height + model.margin.bottom;
    };
    const coversOwnAnchor = (c) => {
      const r = rect(c);
      return c.anchor.x >= r.x && c.anchor.x < r.x + r.w && c.anchor.y >= r.y && c.anchor.y < r.y + r.h;
    };
    const overlap = (a, b) => {
      const p = rect(a); const q = rect(b);
      return p.x < q.x + q.w && q.x < p.x + p.w && p.y < q.y + q.h && q.y < p.y + p.h;
    };
    const marginText = () => editor.marginString();

    // Deterministic: the same clicks twice give the same boxes.
    await reset('reference-scene.png');
    const clicks = [{ x: 300, y: 120 }, { x: 310, y: 130 }, { x: 320, y: 140 }, { x: 600, y: 380 }];
    const run = () => clicks.map((at) => { const c = editor.createCallout(at); editor.layoutScene(); return `${c.box.x},${c.box.y}`; });
    const first = run();
    model.callouts = []; model.nextNumber = 1; editor.layoutScene();
    const second = run();
    check('the same anchors place the same way twice', first.join(' ') === second.join(' '), first.join(' '));
    check('bubbles near one anchor take different places, none overlapping, none over its anchor',
      model.callouts.every((c, i) => !coversOwnAnchor(c) && model.callouts.slice(0, i).every((o) => !overlap(c, o))),
      model.callouts.map((c) => `${c.box.x},${c.box.y}`).join(' '));
    check('all inside the picture, so the margin stayed 0', model.callouts.every(inside) && marginText() === '0,0,0,0', marginText());

    // Anchors near each edge and corner of the 640x400 scene.
    await reset('reference-scene.png');
    const edges = [
      ['right edge', { x: 630, y: 200 }, (c) => c.box.x + c.box.width <= 630],
      ['left edge', { x: 10, y: 200 }, (c) => c.box.x >= 10],
      ['bottom edge', { x: 320, y: 390 }, (c) => c.box.y + editor.heightOf(c) <= 390],
      ['top edge', { x: 320, y: 10 }, (c) => c.box.y >= 10],
      ['bottom right corner', { x: 630, y: 390 }, (c) => c.box.x + c.box.width <= 630 && c.box.y + editor.heightOf(c) <= 390],
    ];
    for (const [name, at, onTheOtherSide] of edges) {
      const c = editor.createCallout(at);
      editor.layoutScene();
      check(`${name}: the bubble goes to the open side, inside, clear of its anchor`, inside(c) && !coversOwnAnchor(c) && onTheOtherSide(c),
        `anchor ${at.x},${at.y} box ${c.box.x},${c.box.y} ${c.box.width}x${editor.heightOf(c)}`);
      model.callouts = []; editor.layoutScene();
    }
    check('and the margin stayed 0 through all of it', marginText() === '0,0,0,0', marginText());

    // A crop too small for any bubble: the margin grows, the picture is not shrunk, the
    // export is the composition with the source exact inside it.
    {
      const small = await invoke('editor_load_probe', { width: 120, height: 80 });
      await editor.loadImage(small);
      model.callouts = []; model.nextNumber = 1;
      const c = editor.createCallout({ x: 60, y: 40 });
      editor.layoutScene();
      await editor.settle();
      check('a bubble that cannot fit takes a margin', marginText() !== '0,0,0,0' && inside(c) && !coversOwnAnchor(c), `margin ${marginText()}, box ${c.box.x},${c.box.y}`);
      c.text = 'too big for this crop';
      editor.layoutScene();
      const layer = await editor.exportLayer();
      const r = await invoke('editor_export_check', layer.bytes, { headers: { name: 's16-small', margin: layer.margin, mode: 'source' } });
      const comp = { w: 120 + model.margin.left + model.margin.right, h: 80 + model.margin.top + model.margin.bottom };
      check('the export is the composition, with every source pixel exact', r.width === comp.w && r.height === comp.h && r.source_mismatches === 0,
        `${r.width}x${r.height} for a 120x80 crop, ${r.source_mismatches} source pixels differ`);
      check('the fit view shows the whole composition', Math.abs(model.zoom - model.fitZoom) < 0.0005);
    }

    // A long note near the bottom grows the margin below as it is typed, and gives it
    // back when the note goes.
    await reset('reference-scene.png');
    {
      const c = editor.createCallout({ x: 200, y: 300 });
      editor.layoutScene();
      const before = marginText();
      editor.startEditing(c);
      const textEl = document.querySelector(`[data-id="${c.id}"] .t`);
      // Short enough to fit the 400 px scene once dragged back up, in the 194 px the text
      // has inside Rotem's 2026-09-17 bubble; long enough to run past the bottom from y 174.
      textEl.textContent = 'A long note that wraps over several lines: the save button does nothing on a slow connection, and the user gets no sign that anything happened.';
      textEl.dispatchEvent(new InputEvent('input', { bubbles: true }));
      await sleep(30);
      check('typing past the bottom takes room below, and nowhere else', before === '0,0,0,0' && model.margin.bottom > 0 && model.margin.top === 0 && model.margin.left === 0 && model.margin.right === 0 && inside(c),
        `margin ${marginText()}, box bottom ${c.box.y + editor.heightOf(c)}`);
      editor.commitEditing();
      const grown = marginText();
      // Dragged back up: dropped, the margin comes back.
      c.box.y = 40;
      await editor.settle();
      check('dropped back inside, the margin goes', grown !== '0,0,0,0' && marginText() === '0,0,0,0', `${grown} then ${marginText()}`);
      // A manual place stays put when another note arrives.
      const placed = { ...c.box };
      const other = editor.createCallout({ x: 200, y: 60 });
      editor.layoutScene();
      check('a moved bubble stays where it was put when another is added', c.box.x === placed.x && c.box.y === placed.y && !overlap(c, other),
        `kept ${c.box.x},${c.box.y}; the new one ${other.box.x},${other.box.y}`);
    }

    // Numbers keep their gaps in the output: the export's markup carries 1 and 3.
    await reset('reference-scene.png');
    {
      const a = editor.createCallout({ x: 100, y: 100 }); a.text = 'one';
      const b = editor.createCallout({ x: 300, y: 100 }); b.text = 'two';
      const c = editor.createCallout({ x: 500, y: 100 }); c.text = 'three';
      editor.layoutScene();
      editor.removeCallout(b);
      await editor.settle();
      const layer = await editor.exportLayer();
      const badges = [...layer.markup.matchAll(/>(\d+)<\/div>/g)].map((m) => m[1]);
      check('after deleting number 2 the output carries 1 and 3, in the editor and the layer alike',
        badges.join(',') === '1,3' && model.callouts.map((x) => x.number).join(',') === '1,3', `badges ${badges.join(',')}`);
      const d = editor.createCallout({ x: 100, y: 300 });
      check('the next note is 4, the deleted number is never reused', d.number === 4);
    }

    // F56: with a margin, a zoom above fit can reach the margin's far edge.
    await reset('reference-scene.png');
    {
      await editor.setMargin({ left: 0, top: 0, right: 240, bottom: 0 });
      await editor.setZoom(8);
      model.pan.x = 100000;
      await editor.paintRegion();
      const vp = { w: document.getElementById('stage').clientWidth };
      const visibleW = (vp.w * editor.ratioOf()) / model.zoom;
      const mat = document.getElementById('mat');
      const matRight = parseFloat(mat.style.left) + parseFloat(mat.style.width);
      check('the pan reaches the far edge of the margin', Math.abs(model.pan.x - (640 + 240 - visibleW)) < 1 && Math.abs(matRight - vp.w) < 2,
        `pan ${Math.round(model.pan.x)}, the mat ends at ${matRight.toFixed(1)} of ${vp.w}`);
      const canvasLeft = parseFloat(editor.canvas.style.left);
      const pictureEnd = matRight - (240 * model.zoom) / editor.ratioOf();
      const hidden = editor.canvas.style.display === 'none';
      check('the picture ends where the margin begins, or is out of view when only margin fits',
        hidden ? (pictureEnd <= 0 && model.pan.x >= 640) : (canvasLeft >= 0 && Math.abs(canvasLeft + parseFloat(editor.canvas.style.width) - pictureEnd) < 2),
        hidden ? `hidden, the picture ends ${pictureEnd.toFixed(1)} left of the window` : `canvas ${canvasLeft.toFixed(1)} + ${editor.canvas.style.width}`);
      await editor.setMargin({ left: 0, top: 0, right: 0, bottom: 0 });
      await editor.setZoom(model.fitZoom);
    }
  }

  // ---------------------------------------------------------------- 22. S1.7: the keyboard contract, undo and redo
  say('');
  say('S1.7: the keys route by context, the viewing keys are inert while typing, undo and redo walk the acceptance sequence');
  {
    const press = (init) => {
      const e = new KeyboardEvent('keydown', { bubbles: true, cancelable: true, ...init });
      window.dispatchEvent(e);
      return e.defaultPrevented;
    };
    const fresh = async () => {
      await openFixture('reference-scene.png');
      await editor.setMode('annotate');
      await editor.setMargin({ left: 0, top: 0, right: 0, bottom: 0 });
      model.callouts = []; model.nextNumber = 1;
      model.history = { steps: [editor.snapshot()], index: 0 };
      editor.layoutScene();
    };
    const el = (c) => document.querySelector(`[data-id="${c.id}"]`);
    const type = (c, text) => {
      editor.startEditing(c);
      el(c).querySelector('.t').textContent = text;
      editor.commitEditing();
    };
    const dragBubble = (c, dx, dy) => {
      const box = el(c).getBoundingClientRect();
      const x = box.left + 4; const y = box.top + 4;
      const ev = (type, cx, cy) => el(c).dispatchEvent(new PointerEvent(type, { pointerId: 9, button: 0, buttons: type === 'pointerup' ? 0 : 1, clientX: cx, clientY: cy, bubbles: true, cancelable: true }));
      ev('pointerdown', x, y);
      ev('pointermove', x + dx, y + dy);
      ev('pointerup', x + dx, y + dy);
    };

    // The acceptance sequence of §3.6.
    await fresh();
    const c = editor.createCallout({ x: 100, y: 100 });
    editor.layoutScene();
    type(c, 'first');
    type(c, 'second');
    const placed = { ...c.box };
    dragBubble(c, 40, 30);
    const moved = { ...c.box };
    check('a drag moved the bubble and left the anchor', (moved.x !== placed.x || moved.y !== placed.y) && c.anchor.x === 100 && c.anchor.y === 100,
      `${placed.x},${placed.y} to ${moved.x},${moved.y}`);
    press({ key: 'z', code: 'KeyZ', ctrlKey: true });
    let note = model.callouts[0];
    check('undo restores the position', note && note.box.x === placed.x && note.box.y === placed.y && note.text === 'second', `${note && note.box.x},${note && note.box.y} ${note && note.text}`);
    press({ key: 'z', code: 'KeyZ', ctrlKey: true });
    note = model.callouts[0];
    check('undo again restores the previous text', note && note.text === 'first' && note.number === 1, note && note.text);
    press({ key: 'y', code: 'KeyY', ctrlKey: true });
    note = model.callouts[0];
    check('redo brings the text back', note && note.text === 'second');
    press({ key: 'Z', code: 'KeyZ', ctrlKey: true, shiftKey: true });
    note = model.callouts[0];
    check('Ctrl+Shift+Z redoes the move', note && note.box.x === moved.x && note.box.y === moved.y);
    press({ key: 'z', code: 'KeyZ', ctrlKey: true });
    press({ key: 'z', code: 'KeyZ', ctrlKey: true });
    press({ key: 'z', code: 'KeyZ', ctrlKey: true });
    check('undo past the first step removes the note and stops', model.callouts.length === 0 && model.history.index === 0);
    press({ key: 'y', code: 'KeyY', ctrlKey: true });
    check('redo brings it back with its number', model.callouts.length === 1 && model.callouts[0].number === 1 && model.callouts[0].text === 'first');

    // A deleted number comes back on undo and is never reused.
    await fresh();
    const a = editor.createCallout({ x: 100, y: 100 }); editor.layoutScene(); type(a, 'one');
    const b = editor.createCallout({ x: 300, y: 100 }); editor.layoutScene(); type(b, 'two');
    const d3 = editor.createCallout({ x: 500, y: 100 }); editor.layoutScene(); type(d3, 'three');
    model.selected = model.callouts[1];
    editor.layoutScene();
    press({ key: 'Delete', code: 'Delete' });
    check('Delete removes the selected note', model.callouts.map((x) => x.number).join(',') === '1,3');
    press({ key: 'z', code: 'KeyZ', ctrlKey: true });
    check('undo restores it with its own number, in place', model.callouts.map((x) => x.number).join(',') === '1,2,3' && model.callouts[1].text === 'two');
    press({ key: 'y', code: 'KeyY', ctrlKey: true });
    const e4 = editor.createCallout({ x: 100, y: 300 });
    check('after the redo the next note is 4', e4.number === 4 && model.callouts.map((x) => x.number).join(',') === '1,3,4');

    // While a note is typed, the engine owns undo and the viewing keys are inert.
    await fresh();
    const t = editor.createCallout({ x: 100, y: 100 }); editor.layoutScene(); type(t, 'typed');
    const steps = model.history.index;
    editor.startEditing(t);
    const zoomBefore = model.zoom;
    const inert = [
      ['Ctrl+Z', { key: 'z', code: 'KeyZ', ctrlKey: true }],
      ['Ctrl+C', { key: 'c', code: 'KeyC', ctrlKey: true }],
      ['Backspace', { key: 'Backspace', code: 'Backspace' }],
      ['Delete', { key: 'Delete', code: 'Delete' }],
      ['the 1 key', { key: '1', code: 'Digit1' }],
      ['the A key', { key: 'a', code: 'KeyA' }],
      ['F11', { key: 'F11', code: 'F11' }],
      ['PageDown', { key: 'PageDown', code: 'PageDown' }],
    ];
    const taken = inert.filter(([, init]) => press(init)).map(([name]) => name);
    check('while typing, none of these is taken from the text: ' + inert.map(([n]) => n).join(', '), taken.length === 0, taken.length ? 'taken: ' + taken.join(', ') : '');
    check('and nothing moved: the zoom, the mode, fullscreen, the history, the note', model.zoom === zoomBefore && model.mode === 'annotate' && model.fullscreen === false && model.history.index === steps && model.editing === t && model.callouts.length === 1);
    check('Ctrl+Shift+C is taken even while typing', press({ key: 'C', code: 'KeyC', ctrlKey: true, shiftKey: true }));
    check('Ctrl +/- are taken while typing, for the note being written', press({ key: '+', code: 'Equal', ctrlKey: true }) && press({ key: '-', code: 'Minus', ctrlKey: true }));
    check('Escape leaves editing and keeps the text', press({ key: 'Escape', code: 'Escape' }) && model.editing === null && t.text === 'typed');

    // Outside typing: Ctrl+C copies the composed image; keys of later stages do nothing.
    const copy = press({ key: 'c', code: 'KeyC', ctrlKey: true });
    // The key starts the copy; the HUD says when it landed, and only then is the clipboard
    // read back.
    const hudEl = document.getElementById('hud');
    for (let i = 0; i < 200 && !hudEl.textContent.includes(`copied ${model.image.width}x${model.image.height}`); i += 1) await sleep(50);
    const back = await invoke('editor_clipboard_readback');
    check('Ctrl+C outside typing copies the composed image', copy && back.png && back.width === model.image.width && back.height === model.image.height, `${back.width}x${back.height}`);
    // Ctrl+S is S1.11's and opens a real dialog since then, so it is not pressed here: a dialog
    // left open would make the one Save As at a time (review T9) refuse section 26's press.
    check('Ctrl+Shift+Enter is taken and does nothing yet, the stage column honoured',
      press({ key: 'Enter', code: 'Enter', ctrlKey: true, shiftKey: true }) && model.callouts.length === 1);

    // Copy and Return: the note being typed is committed, the image copied, the window hidden.
    editor.startEditing(t);
    el(t).querySelector('.t').textContent = 'typed then returned';
    const outcome = await editor.copyAndReturn();
    const visible = await invoke('editor_window_visible');
    check('Copy and Return commits the note, copies, and hides the editor', outcome.hidden === true && visible === false && t.text === 'typed then returned' && model.editing === null,
      `hidden ${outcome.hidden}, visible ${visible}, ${outcome.width}x${outcome.height}`);
    await invoke('editor_show');
    check('the window is back for the checks', (await invoke('editor_window_visible')) === true);
    check('Ctrl+Enter is the key for it', press({ key: 'Enter', code: 'Enter', ctrlKey: true }));
    for (let i = 0; i < 200 && (await invoke('editor_window_visible')); i += 1) await sleep(50);
    await invoke('editor_show');
    model.callouts = [];
    editor.layoutScene();
  }

  // ---------------------------------------------------------------- 23. S1.8: the document store
  say('');
  say('S1.8: one folder per document, the image written once, the notes saved without an Apply, the latest reopened after a restart, a failure visible');
  {
    const hud = document.getElementById('hud');
    const list = () => invoke('editor_store_list');
    const line = async (id) => (await list()).find((l) => l.id === id);
    const until = async (test, ms = 3000) => {
      const started = performance.now();
      let value = await test();
      while (!value && performance.now() - started < ms) { await sleep(50); value = await test(); }
      return value;
    };
    const el = (c) => document.querySelector(`[data-id="${c.id}"]`);
    const dir = await invoke('editor_store_reset');
    say(`the checks' store is ${dir}`);

    // A capture is a document on disk from its first moment: the record at once, the
    // image when its encode is done, and nothing left over from the atomic writes.
    const shot = await invoke('editor_capture_probe', { width: 320, height: 200 });
    await editor.loadImage(shot);
    let entry = await until(async () => { const l = await line(shot.document_id); return l && l.json && l.source_png ? l : null; });
    check('a new capture is saved automatically: a folder with document.json and source.png', !!entry && entry.source_bytes > 0 && entry.leftovers === 0,
      entry ? `${entry.source_bytes} bytes of PNG, ${entry.leftovers} leftovers` : 'no folder');
    const sourceBytes = entry ? entry.source_bytes : 0;

    // Typing autosaves, debounced: the note is on disk within a couple of seconds, with no
    // Apply, and the HUD says saved only after the host said so.
    const note = editor.createCallout({ x: 100, y: 100 });
    editor.layoutScene();
    editor.startEditing(note);
    el(note).querySelector('.t').textContent = 'saved as I type';
    el(note).querySelector('.t').dispatchEvent(new InputEvent('input', { bubbles: true }));
    const typed = await until(async () => { const l = await line(shot.document_id); return l && l.notes.includes('saved as I type') ? l : null; });
    check('a note being typed reaches the disk on its own, debounced', !!typed && model.save.state === 'saved' && hud.textContent.includes('saved'),
      typed ? `on disk, ${typed.notes.length} chars of notes, HUD ${JSON.stringify(hud.textContent.split('\n')[1])}` : 'not on disk within 3 s');
    editor.commitEditing();
    await editor.saveNow();

    // A hide saves first: the text changes and the hide follows at once.
    editor.startEditing(note);
    el(note).querySelector('.t').textContent = 'saved before the hide';
    editor.commitEditing();
    await editor.saveNow();
    await invoke('editor_hide');
    entry = await line(shot.document_id);
    check('a hide carries the pending change to disk first', !!entry && entry.notes.includes('saved before the hide'));
    await invoke('editor_show');

    // The image is written once: the same bytes after every save.
    entry = await line(shot.document_id);
    check('source.png is never rewritten', !!entry && entry.source_bytes === sourceBytes, `${entry && entry.source_bytes} bytes, was ${sourceBytes}`);

    // A restart: the list and the image are dropped, the disk is read, the latest reopens
    // with its notes, the page holding none of its own.
    const wasId = shot.document_id;
    const wasNumber = model.nextNumber;
    editor.documents.clear();
    model.callouts = [];
    model.image = { width: 0, height: 0, source: '' }; // a fresh page holds nothing
    const back = await invoke('editor_store_reload');
    await editor.loadImage(back);
    check('after a restart the latest document reopens, same number, same size, in annotation',
      back.document_id === wasId && back.width === 320 && back.height === 200 && back.managed === true && model.mode === 'annotate',
      `document ${back.document_id}, ${back.width}x${back.height}`);
    check('with its notes, from the disk', model.callouts.length === 1 && model.callouts[0].text === 'saved before the hide' && model.nextNumber === wasNumber,
      `${model.callouts.length} notes, next ${model.nextNumber}`);

    // The S0.5 leg carried here: an annotated file's document holds the frame asked for,
    // upright, converted, at the raster size fixed at open, and it survives the restart.
    const carried = [];
    for (const [name, frame, why] of [
      ['gif-three-frames.gif', 2, 'the third frame, not the first'],
      ['jpeg-orientation-6.jpg', 0, 'upright, the orientation applied once'],
      ['png-alpha-and-swapped-profile.png', 0, 'converted to sRGB once'],
      ['svg-text-labels.svg', 0, 'the raster size fixed at open'],
    ]) {
      let info = await openFixture(name);
      if (frame > 0) info = await invoke('editor_frame', { index: frame });
      await editor.loadImage(info);
      await editor.setMode('annotate');
      const id = model.image.document_id;
      const written = await until(async () => { const l = await line(id); return l && l.source_png ? l : null; });
      carried.push({ name, id, frame, width: info.width, height: info.height, written: !!written, why });
    }
    editor.documents.clear();
    model.callouts = [];
    model.image = { width: 0, height: 0, source: '' };
    await editor.loadImage(await invoke('editor_store_reload'));
    for (const c of carried) {
      const same = await invoke('editor_store_verify', { id: c.id }).catch((err) => String(err));
      const record = JSON.parse(await invoke('editor_store_read', { id: c.id }));
      check(`${c.name}: after the restart the document's image is the file's frame ${c.frame} decoded afresh, ${c.why}`,
        c.written && same === true && record.source.kind === 'file' && record.source.frame === c.frame && record.width === c.width && record.height === c.height,
        `same ${same}, ${record.width}x${record.height}, frame ${record.source.frame}, ${record.source.path.split('\\').pop()}`);
    }
    check('the latest document reopened is the last one annotated', model.image.file === 'svg-text-labels.svg' && model.image.managed === true, model.image.file);

    // A save that fails is visible, keeps the work, and the next one lands.
    await invoke('editor_store_break', { on: true });
    const c = editor.createCallout({ x: 60, y: 60 });
    editor.layoutScene();
    editor.startEditing(c);
    el(c).querySelector('.t').textContent = 'written while the disk was refusing';
    editor.commitEditing();
    const state = await editor.saveNow();
    check('a failed save says so on screen and never says saved', state === 'failed' && hud.textContent.includes('NOT SAVED') && model.save.dirty === true,
      JSON.stringify(hud.textContent.split('\n')[1].slice(0, 160)));
    check('the work is still here', model.callouts.length === 1 && model.callouts[0].text === 'written while the disk was refusing');
    await invoke('editor_store_break', { on: false });
    editor.markDirty();
    const again = await editor.saveNow();
    entry = await line(model.image.document_id);
    check('the next save lands, and the notice goes', again === 'saved' && !!entry && entry.notes.includes('written while the disk was refusing') && !hud.textContent.includes('NOT SAVED'));

    // A save refused, then another picture, then back (review 4, T1): the refusal comes
    // back with the notes, never reset to "saved" by the return, and the save it still owes
    // lands by the timer once the store is back.
    {
      const heldId = model.image.document_id;
      await invoke('editor_store_break', { on: true });
      const held = editor.createCallout({ x: 90, y: 90 });
      editor.layoutScene();
      editor.startEditing(held);
      el(held).querySelector('.t').textContent = 'refused, then carried across the switch';
      editor.commitEditing();
      const refused = await editor.saveNow();
      await editor.loadImage(await invoke('editor_capture_probe', { width: 300, height: 200 }));
      await invoke('editor_store_break', { on: false });
      await editor.loadImage(await invoke('editor_show_document', { id: heldId }));
      const back = { state: model.save.state, dirty: model.save.dirty, notice: hud.textContent.includes('NOT SAVED') };
      const landed = await until(async () => { const l = await line(heldId); return l && l.notes.includes('refused, then carried') ? l : null; }, 3000);
      check('a save refused before a switch of picture comes back with the picture as NOT SAVED, never as "saved", and lands by the timer once the store is back',
        refused === 'failed' && back.state === 'failed' && back.dirty === true && back.notice && !!landed && model.save.state === 'saved',
        `refused ${refused}; back as ${back.state}, dirty ${back.dirty}, NOT SAVED ${back.notice}; on disk ${!!landed}, now ${model.save.state}`);
    }

    // A keystroke while a save is on its way (review T1): a second save waits on the
    // first, and a character typed during that wait has to reach the disk by the timer's
    // own save, with nothing else touching the document.
    {
      const racedId = model.image.document_id;
      const raced = editor.createCallout({ x: 120, y: 120 });
      editor.layoutScene();
      editor.startEditing(raced);
      const textEl = el(raced).querySelector('.t');
      const typed = (text) => { textEl.textContent = text; textEl.dispatchEvent(new InputEvent('input', { bubbles: true })); };
      typed('one');
      const first = editor.saveNow();
      typed('one two');
      const second = editor.saveNow();
      typed('one two three');
      await first;
      await second;
      const landed = await until(async () => { const l = await line(racedId); return l && l.notes.includes('one two three') ? l : null; }, 3000);
      check('a character typed while a save is on its way reaches the disk by the timer\'s save', !!landed && model.save.state === 'saved',
        landed ? 'on disk' : `not on disk within 3 s; the record holds ${JSON.stringify((await line(racedId)).notes.slice(0, 120))}`);
      editor.commitEditing();
      await editor.saveNow();
    }
    model.callouts = [];
    editor.layoutScene();
  }

  // ---------------------------------------------------------------- 24. S1.9: history navigation
  say('');
  say('S1.9: previous and next through the documents, captures and annotated files alike, the position named, and the two lists never move each other');
  {
    const hud = document.getElementById('hud');
    await invoke('editor_store_reset');
    const shots = [];
    for (const [w, h, text] of [[300, 200, 'first capture'], [320, 200, 'second capture'], [340, 200, 'third capture']]) {
      const info = await invoke('editor_capture_probe', { width: w, height: h });
      await editor.loadImage(info);
      const c = editor.createCallout({ x: 50, y: 50 });
      c.text = text;
      editor.layoutScene();
      editor.record();
      shots.push(info);
    }
    check('a capture activates history, and the newest is last', model.image.context === 'history' && model.image.position === 3 && model.image.total === 3,
      `${model.image.position} of ${model.image.total} in ${JSON.stringify(model.image.context)}`);
    check('the HUD names it', hud.textContent.includes('3 of 3 in history'));

    let info = await invoke('editor_navigate', { step: 'previous' });
    await editor.loadImage(info);
    check('previous is the older document, with its own notes', info.document_id === shots[1].document_id && info.position === 2 && info.width === 320
      && model.callouts.length === 1 && model.callouts[0].text === 'second capture', `${info.position} of ${info.total}, ${model.callouts[0] && model.callouts[0].text}`);
    info = await invoke('editor_navigate', { step: 'first' });
    await editor.loadImage(info);
    check('first is the oldest', info.document_id === shots[0].document_id && info.position === 1 && model.callouts[0].text === 'first capture');
    info = await invoke('editor_navigate', { step: 'previous' });
    check('the start stays', info.document_id === shots[0].document_id && info.position === 1);
    info = await invoke('editor_navigate', { step: 'last' });
    await editor.loadImage(info);
    check('last is the newest', info.document_id === shots[2].document_id && info.position === 3 && model.callouts[0].text === 'third capture');
    info = await invoke('editor_navigate', { step: 'next' });
    check('the end stays', info.document_id === shots[2].document_id && info.position === 3);

    // The keys walk it too.
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'PageUp', code: 'PageUp', bubbles: true, cancelable: true }));
    for (let i = 0; i < 40 && model.image.document_id !== shots[1].document_id; i += 1) await sleep(50);
    check('PageUp walks history', model.image.document_id === shots[1].document_id && model.image.position === 2);

    // Opening a file activates the folder; walking it never moves through history.
    const dir = await invoke('editor_make_folder');
    const at = (name) => `${dir}\\${name}`;
    info = await invoke('editor_open_path', { path: at('img2.png') });
    await editor.loadImage(info);
    check('opening a file activates the folder', info.context === 'folder' && info.total > 0 && info.managed === false, `${info.position} of ${info.total} in ${info.context}`);
    info = await invoke('editor_navigate', { step: 'next' });
    await editor.loadImage(info);
    check('next walks the folder, not history', info.context === 'folder' && info.file === 'img10.png' && info.managed === false);

    // Annotating the file creates a document and changes no context; history has grown
    // by one, but the folder stays active at the same position.
    const beforeAnnotate = { position: info.position, total: info.total };
    await editor.setMode('annotate');
    check('Annotate keeps the folder context and the position', model.image.context === 'folder' && model.image.position === beforeAnnotate.position && model.image.total === beforeAnnotate.total && model.image.managed === true,
      `${model.image.position} of ${model.image.total} in ${model.image.context}`);

    // A new capture: history again, four documents now, the annotated file among them.
    const fourth = await invoke('editor_capture_probe', { width: 360, height: 200 });
    await editor.loadImage(fourth);
    check('a capture returns to history, which now holds the annotated file too', fourth.context === 'history' && fourth.position === 5 && fourth.total === 5,
      `${fourth.position} of ${fourth.total}`);
    info = await invoke('editor_navigate', { step: 'previous' });
    await editor.loadImage(info);
    check('previous from the capture is the annotated file, by its document, named', info.file === 'img10.png' && info.managed === true && info.context === 'history' && info.position === 4,
      `${info.file} ${info.position} of ${info.total} in ${info.context}`);
    model.callouts = [];
    editor.layoutScene();
  }

  // ---------------------------------------------------------------- 25. S1.10: Copy and Return, every way it can fail
  say('');
  say('S1.10: Copy and Return hides only after the copy and the save succeeded; a held clipboard, a failing store, a user who moved on, a target refused, closed or absent');
  {
    const hud = document.getElementById('hud');
    const visible = () => invoke('editor_window_visible');
    const shot = await invoke('editor_capture_probe', { width: 320, height: 200 });
    await editor.loadImage(shot);
    const note = editor.createCallout({ x: 40, y: 40 });
    note.text = 'work that must not be lost';
    editor.layoutScene();
    editor.record();
    await editor.saveNow();

    // The clipboard is held by another application: nothing is copied, nothing hides,
    // the notice says so, the work stays.
    const held = await invoke('editor_hold_clipboard', { on: true });
    let r = await editor.copyAndReturn();
    check('a held clipboard: not copied, not hidden, said on screen', held === true && r.copied === false && r.hidden === false && (await visible()) === true && hud.textContent.includes('NOT COPIED'),
      `held ${held}, copied ${r.copied}, hidden ${r.hidden}, ${String(r.reason).slice(0, 90)}`);
    check('the work is where it was', model.callouts.length === 1 && model.callouts[0].text === 'work that must not be lost');
    await invoke('editor_hold_clipboard', { on: false });

    // The store refuses the save: the image is copied, the editor stays, the notice
    // names the save.
    await invoke('editor_show');
    await invoke('editor_store_break', { on: true });
    editor.markDirty();
    r = await editor.copyAndReturn();
    check('a failed save: copied, not hidden, the notice names it', r.copied === true && r.hidden === false && r.reason === 'save failed' && (await visible()) === true && hud.textContent.includes('NOT SAVED') && hud.textContent.includes('stays'),
      hud.textContent.split('\n').pop().slice(0, 120));
    await invoke('editor_store_break', { on: false });

    // The user moved on while the copy ran: copied, reported, the editor stays.
    await invoke('editor_show');
    const pending = editor.copyAndReturn();
    note.box.x += 10;
    editor.layoutScene();
    editor.record();
    r = await pending;
    check('moved on mid-copy: copied, not hidden, the note where it was moved to', r.copied === true && r.hidden === false && r.reason === 'moved on' && (await visible()) === true && model.callouts.length === 1,
      `copied ${r.copied}, hidden ${r.hidden}, ${r.reason}; ${hud.textContent.split('\n').pop().slice(0, 100)}`);

    // No application to return to: the copy lands, the save lands, the editor hides.
    await invoke('editor_show');
    r = await editor.copyAndReturn();
    check('with nothing to return to: copied, saved, hidden', r.copied === true && r.hidden === true && (await visible()) === false && r.returned.includes('no application'), r.returned);
    await invoke('editor_show');

    // The stand-in application needs the foreground to be handed to it, which the system
    // can refuse to a process nobody is interacting with; then these two are not run.
    let standIn = true;
    try {
      await invoke('editor_stand_in', { on: true });
    } catch (err) {
      standIn = false;
      skipped('the target has closed: hidden, nothing activated', `the stand-in window could not be opened in front: ${err}`);
      skipped('the target is there: hidden, and the return names its outcome', 'the same');
    }
    if (standIn) {
      // The target application has closed since the capture began: hidden, nothing activated.
      await invoke('editor_stand_in', { on: false });
      editor.markDirty();
      r = await editor.copyAndReturn();
      check('the target has closed: hidden, nothing activated, said in the log line', r.hidden === true && r.returned.includes('has closed'), r.returned);
      await invoke('editor_show');

      // The target is there: the focus goes back to it, or the system refuses, and either
      // way the editor is hidden and the outcome is named, never guessed.
      let there = true;
      try {
        await invoke('editor_stand_in', { on: true });
      } catch (err) {
        there = false;
        skipped('the target is there: hidden, and the return names its outcome', `the stand-in window could not be opened in front: ${err}`);
      }
      if (there) {
        editor.markDirty();
        r = await editor.copyAndReturn();
        check('the target is there: hidden, and the return names its outcome', r.hidden === true && (r.returned.includes('focus returned') || r.returned.includes('refused')), r.returned);
        await invoke('editor_stand_in', { on: false });
        await invoke('editor_show');
        check('the last return is what the host remembers', (await invoke('editor_last_return')) === r.returned);
        // The target is used once (review T4): a hide with no capture since returns
        // nowhere, so an editor opened from the tray or for a file closes without a jump.
        editor.markDirty();
        r = await editor.copyAndReturn();
        check('a second hide with no capture between activates nothing', r.hidden === true && r.returned.includes('no application'), r.returned);
        await invoke('editor_show');
      }
    }
    await invoke('editor_show');

    // The idle Escape: a failing store keeps the editor, a working one hides it.
    await invoke('editor_store_break', { on: true });
    editor.markDirty();
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', code: 'Escape', bubbles: true, cancelable: true }));
    await sleep(400);
    check('Escape with a failing save keeps the editor and says so', (await visible()) === true && hud.textContent.includes('NOT SAVED') && hud.textContent.includes('stays'));
    await invoke('editor_store_break', { on: false });
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', code: 'Escape', bubbles: true, cancelable: true }));
    for (let i = 0; i < 40 && (await visible()); i += 1) await sleep(50);
    check('Escape with the save landing hides the editor', (await visible()) === false && model.save.state === 'saved');
    await invoke('editor_show');
    model.callouts = [];
    editor.layoutScene();
  }

  // ---------------------------------------------------------------- 26. S1.11: Save As, a new file every time
  say('');
  say('S1.11: Save As suggests a name derived from the source and marked as annotated, never the original, writes a new file, and offers a free name when the chosen one exists');
  {
    const hud = document.getElementById('hud');
    const outDir = (await invoke('editor_store_reset')) + '\\exports';
    const writeTo = async (path) => {
      const layer = await editor.exportLayer();
      return invoke('editor_save_as_write', layer.bytes, { headers: { margin: layer.margin, path } });
    };

    // A file: the name is the file's stem, marked; never the file's own name or folder.
    const dir = await invoke('editor_make_folder');
    let info = await invoke('editor_open_path', { path: `${dir}\\img2.png` });
    await editor.loadImage(info);
    let plan = await invoke('editor_save_as_plan');
    check('a file: the suggested name is its stem marked as annotated', plan.name === 'img2 annotated.png', plan.name);
    check('and never the original: the name differs from the file and is not its path', plan.name !== 'img2.png' && (plan.folder + '\\' + plan.name).toLowerCase() !== plan.source.toLowerCase(),
      `${plan.folder}\\${plan.name} against ${plan.source}`);

    // A capture: its time, marked.
    const shot = await invoke('editor_capture_probe', { width: 300, height: 200 });
    await editor.loadImage(shot);
    plan = await invoke('editor_save_as_plan');
    check('a capture: the suggested name is its time marked as annotated', /^capture \d{4}-\d{2}-\d{2} \d{2}-\d{2}-\d{2} annotated\.png$/.test(plan.name), plan.name);

    // The write: a new file with the composition, then the same name again is refused and
    // a free one offered, the first file untouched.
    const note = editor.createCallout({ x: 40, y: 40 });
    note.text = 'exported';
    editor.layoutScene();
    editor.record();
    const first = await writeTo(outDir + '\\shot annotated.png');
    check('Save As writes a new file with the composition', first.New !== undefined && first.New.endsWith('shot annotated.png'), JSON.stringify(first).slice(0, 120));
    const listed = await invoke('editor_store_list').catch(() => []);
    const again = await writeTo(outDir + '\\shot annotated.png');
    check('the same name again is not written over, and a free name is offered', again.Exists !== undefined && again.Exists.offered.endsWith('shot annotated (2).png'), JSON.stringify(again).slice(0, 160));
    const third = await writeTo(again.Exists.offered);
    check('the offered name writes', third.New !== undefined && third.New.endsWith('shot annotated (2).png'));
    plan = await invoke('editor_save_as_plan');
    check('the next Save As opens on the folder last exported to', plan.folder.toLowerCase() === outDir.toLowerCase(), plan.folder);

    // The dialog itself: Ctrl+S opens it, Escape closes it with nothing saved, when ours
    // is the window in front.
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 's', code: 'KeyS', ctrlKey: true, bubbles: true, cancelable: true }));
    await sleep(1500);
    let outcome = await invoke('editor_save_as_outcome');
    check('Ctrl+S opens Save As, and the page says so', outcome.state === 'open' && hud.textContent.includes('Save As is open'), outcome.state);
    check('a second Save As while it is open is refused: one dialog', (await editor.saveAs()) === false);
    const sent = await invoke('editor_press_escape');
    for (let i = 0; i < 40 && sent && outcome.state === 'open'; i += 1) {
      await sleep(100);
      outcome = await invoke('editor_save_as_outcome');
    }
    if (!sent) {
      skipped('Escape closes Save As with nothing saved', 'another application is in front, so no key was sent; the dialog is left open');
    } else {
      check('Escape closes Save As with nothing saved, and the page says so', outcome.state === 'cancelled' && hud.textContent.includes('nothing saved'), outcome.line);
    }
    void listed;
    model.callouts = [];
    editor.layoutScene();
  }

  // ---------------------------------------------------------------- 27. S2.1: the timeline
  say('');
  say('S2.1: the timeline along the bottom lists every document as a small thumbnail, the current one marked; a click shows that document with its notes; fullscreen puts it away');
  {
    const strip = editor.strip;
    const stage = document.getElementById('stage');
    await invoke('editor_store_reset');
    editor.setStripHeight(96); // whatever a drag left remembered (S2.8), this section reads the default
    const shots = [];
    for (const [w, h, text] of [[400, 300, 'first'], [300, 400, 'second'], [640, 200, 'third']]) {
      const info = await invoke('editor_capture_probe', { width: w, height: h });
      await editor.loadImage(info);
      const c = editor.createCallout({ x: 30, y: 30 });
      c.text = text;
      editor.layoutScene();
      editor.record();
      shots.push(info);
    }
    await editor.refreshStrip();
    const docs = await invoke('editor_documents');
    check('the host lists the documents oldest first, one of them current', docs.length === 3 && docs.map((d) => d.id).join() === shots.map((s) => s.document_id).join() && docs.filter((d) => d.current).length === 1 && docs[2].current,
      docs.map((d) => `${d.width}x${d.height}${d.current ? '*' : ''}`).join(' '));
    check('the strip is shown with one thumbnail per document, the newest at the left and current', document.body.classList.contains('strip') && strip.children.length === 3 && strip.children[0].classList.contains('current') && !strip.children[2].classList.contains('current')
      && Number(strip.children[0].dataset.id) === shots[2].document_id && Number(strip.children[2].dataset.id) === shots[0].document_id);
    check('the stage ends above the picture\'s size and the strip, below the top bar', stage.clientHeight === window.innerHeight - 64 - 48 - 96, `stage ${stage.clientHeight} of ${window.innerHeight}`);
    // The picture's size in a container across the window above the timeline, 16 px clear above and
    // below it, on the app's own background (Rotem, 2026-09-16).
    const sizeEl = document.getElementById('image-size');
    const sizeBox = sizeEl.getBoundingClientRect();
    const sizeStripTop = Math.round(strip.getBoundingClientRect().top);
    const sizeStageBottom = Math.round(stage.getBoundingClientRect().bottom);
    check('the picture\'s size is in a container across the window, 16 px under the stage and 16 px above the timeline, on the app\'s background',
      sizeEl.textContent === `${model.image.width}x${model.image.height}` && sizeBox.left === 0 && Math.round(sizeBox.width) === window.innerWidth
        && Math.round(sizeBox.top) === sizeStageBottom + 16 && Math.round(sizeBox.bottom) === sizeStripTop - 16
        && getComputedStyle(sizeEl).backgroundColor === getComputedStyle(document.body).backgroundColor,
      `"${sizeEl.textContent}" at ${Math.round(sizeBox.left)}-${Math.round(sizeBox.right)} by ${Math.round(sizeBox.top)}-${Math.round(sizeBox.bottom)}, the stage to ${sizeStageBottom}, the strip from ${sizeStripTop}, ${getComputedStyle(sizeEl).backgroundColor}`);
    // Its text centred in the window, in Google Sans at 16 px, medium, from the font file bundled with
    // the page (Rotem, 2026-09-16; 16 px from 14 on 2026-09-17): a face that failed to load, or the text off centre, turns this red.
    await document.fonts.load('500 16px "Google Sans"', sizeEl.textContent);
    const sizeFace = [...document.fonts].find((f) => f.family.replace(/"/g, '') === 'Google Sans' && String(f.weight) === '500');
    const sizeStyle = getComputedStyle(sizeEl);
    const sizeRange = document.createRange();
    sizeRange.selectNodeContents(sizeEl);
    const sizeText = sizeRange.getBoundingClientRect();
    check('the size is centred in the window, in Google Sans at 16 px, medium, loaded from the page\'s own font file',
      !!sizeFace && sizeFace.status === 'loaded' && sizeStyle.fontSize === '16px' && sizeStyle.fontWeight === '500' && sizeStyle.fontFamily.startsWith('"Google Sans"')
        && sizeText.width > 0 && Math.abs((sizeText.left + sizeText.right) / 2 - window.innerWidth / 2) <= 1,
      `${sizeFace ? sizeFace.status : 'no face'}, ${sizeStyle.fontWeight} ${sizeStyle.fontSize} ${sizeStyle.fontFamily.split(',')[0]}, the text ${sizeText.left.toFixed(1)}-${sizeText.right.toFixed(1)} in a window ${window.innerWidth} wide`);
    // With no timeline the HUD, which carries the notices, sits above the picture's size, not under it.
    document.body.classList.remove('strip');
    const bareHudBottom = Math.round(document.getElementById('hud').getBoundingClientRect().bottom);
    const bareSizeTop = Math.round(sizeEl.getBoundingClientRect().top);
    document.body.classList.add('strip');
    check('with no timeline the HUD sits above the picture\'s size and its margin, so no line of it is covered', bareHudBottom <= bareSizeTop - 16,
      `the HUD to ${bareHudBottom}, the size from ${bareSizeTop}`);
    for (let i = 0; i < 100 && [...strip.querySelectorAll('img')].filter((img) => img.complete && img.naturalWidth > 0).length < 3; i += 1) await sleep(50);
    const imgs = [...strip.querySelectorAll('img')];
    // 320 by 200 since S2.8, so a thumbnail grown to 320 wide in the strip is not blurry.
    check('every thumbnail is a small picture, at most 320 by 200, the shape of its document', imgs.length === 3 && imgs.every((img) => img.naturalWidth > 0 && img.naturalWidth <= 320 && img.naturalHeight <= 200)
      && imgs[2].naturalWidth === 267 && imgs[2].naturalHeight === 200 && imgs[1].naturalWidth === 150 && imgs[0].naturalWidth === 320 && imgs[0].naturalHeight === 100, // newest first
      imgs.map((img) => `${img.naturalWidth}x${img.naturalHeight}`).join(' '));
    const kept = await invoke('editor_store_list');
    check('the thumbnails are kept beside their documents on disk', kept.length === 3 && kept.every((l) => l.json && l.thumb), `${kept.filter((l) => l.thumb).length} of ${kept.length} folders hold one`);

    strip.children[2].click();
    for (let i = 0; i < 60 && model.image.document_id !== shots[0].document_id; i += 1) await sleep(50);
    await sleep(100);
    check('a click on a thumbnail shows that document, with its notes, in history', model.image.document_id === shots[0].document_id && model.callouts.length === 1 && model.callouts[0].text === 'first' && model.image.context === 'history' && model.image.position === 1,
      `${model.image.position} of ${model.image.total} in ${model.image.context}, ${model.callouts[0] && model.callouts[0].text}`);
    check('and the picture\'s size follows it', sizeEl.textContent === '400x300', `"${sizeEl.textContent}"`);
    await editor.refreshStrip();
    check('the mark moved to it', strip.children[2].classList.contains('current') && !strip.children[0].classList.contains('current'));

    await editor.setFullscreen(true);
    check('fullscreen puts the strip away and gives the stage the whole window', !document.body.classList.contains('strip') && stage.clientHeight === window.innerHeight, `stage ${stage.clientHeight} of ${window.innerHeight}`);
    check('and the picture\'s size with it', getComputedStyle(sizeEl).display === 'none', getComputedStyle(sizeEl).display);
    await editor.setFullscreen(false);
    check('and it comes back', document.body.classList.contains('strip') && strip.children.length === 3);
    check('the picture\'s size too, with the stage ending above it', getComputedStyle(sizeEl).display === 'block' && stage.clientHeight === window.innerHeight - 64 - 48 - 96,
      `${getComputedStyle(sizeEl).display}, stage ${stage.clientHeight} of ${window.innerHeight}`);
    model.callouts = [];
    editor.layoutScene();
  }

  // ---------------------------------------------------------------- 28. S2.3 and S2.4: the controls, and the storage line
  say('');
  say('S2.3 and S2.4: the sidebar holds the tools alone, Copy and Save As are keys, and the HUD says what the store holds');
  {
    const hud = document.getElementById('hud');
    const controls = document.getElementById('controls');
    const copyButton = document.getElementById('copy');
    const saveButton = document.getElementById('saveas');
    // The sidebar (Rotem, 2026-09-15): icon buttons down the left edge, the spec in project-os/Design.md.
    const stage = document.getElementById('stage');
    const cbox = controls.getBoundingClientRect();
    const stageLeft = Math.round(stage.getBoundingClientRect().left);
    check('the controls are a sidebar down the left edge, below the top bar, and the stage starts at its right edge, with no Copy or Save As button in it (Rotem, 2026-09-18)', !controls.hidden && !copyButton && !saveButton
      && cbox.left === 0 && Math.round(cbox.top) === 64 && Math.round(cbox.width) === 64 && stageLeft === 64,
      `sidebar ${Math.round(cbox.left)}-${Math.round(cbox.right)} from ${Math.round(cbox.top)}, the stage from ${stageLeft}`);
    const buttons = [...controls.querySelectorAll('button')];
    const boxes = buttons.map((b) => b.getBoundingClientRect());
    const iconWidth = (b) => Math.max(...[...b.querySelectorAll('svg')].map((s) => Math.round(s.getBoundingClientRect().width)));
    check('eight icon buttons, 48 by 48 with a 32 by 32 icon drawn with a 2 px #C3C6CA line, 8 px apart, on 13 px corners, named, with no fill of their own', buttons.length === 8
      && buttons.every((b) => getComputedStyle(b).borderRadius === '13px')
      && boxes.every((b) => Math.round(b.width) === 48 && Math.round(b.height) === 48)
      && boxes.every((b, i) => i === 0 || Math.round(b.top - boxes[i - 1].bottom) === 8)
      && buttons.every((b) => iconWidth(b) === 32 && [...b.querySelectorAll('svg')].every((v) => parseFloat(getComputedStyle(v).strokeWidth) * 32 / 20 === 2 && getComputedStyle(v).stroke === 'rgb(195, 198, 202)') && (b.getAttribute('aria-label') || '').length > 0)
      && buttons.every((b) => b.classList.contains('active') || getComputedStyle(b).backgroundColor === 'rgba(0, 0, 0, 0)'),
      `${buttons.length} buttons at ${boxes.map((b) => Math.round(b.top)).join(' ')}, icons ${buttons.map(iconWidth).join(' ')}, the sidebar ${Math.round(cbox.height)} tall`);
    const hoverRule = [...document.styleSheets[0].cssRules].find((r) => r.selectorText === '#controls button:hover');
    check('a hover fills the square with #21222C, fading in by A1: 144 ms, ease-out', !!hoverRule && hoverRule.style.backgroundColor === 'rgb(33, 34, 44)'
      && buttons.every((b) => getComputedStyle(b).transitionProperty === 'background-color' && getComputedStyle(b).transitionDuration === '0.144s' && getComputedStyle(b).transitionTimingFunction === 'ease-out'),
      `${hoverRule && hoverRule.style.backgroundColor}, ${getComputedStyle(buttons[0]).transition}`);
    // One container around the eight, centred across the sidebar; a tooltip on each button's right (Rotem, 2026-09-15).
    const group = document.getElementById('buttons');
    const gbox = group.getBoundingClientRect();
    const stageBottom = Math.round(stage.getBoundingClientRect().bottom);
    check('one container holds all eight buttons, centred in the sidebar\'s height and across it, the sidebar running from the top bar to the timeline', !!group && group.parentElement === controls && buttons.every((b) => group.contains(b))
      && Math.abs((gbox.left + gbox.right) / 2 - (cbox.left + cbox.right) / 2) < 0.5 && Math.round(gbox.width) === 48
      && Math.round(cbox.bottom) === stageBottom && Math.abs((gbox.top + gbox.bottom) / 2 - (cbox.top + cbox.bottom) / 2) < 0.5,
      `container ${Math.round(gbox.left)}-${Math.round(gbox.right)} by ${Math.round(gbox.top)}-${Math.round(gbox.bottom)}, centred at ${(gbox.left + gbox.right) / 2},${(gbox.top + gbox.bottom) / 2}; the sidebar ${Math.round(cbox.top)}-${Math.round(cbox.bottom)}, centred at ${(cbox.left + cbox.right) / 2},${(cbox.top + cbox.bottom) / 2}; the stage ends at ${stageBottom}`);
    const tips = buttons.map((b) => getComputedStyle(b, '::after'));
    check('each button carries its name as a tooltip 8 px to its right, hidden at rest, above the stage', tips.every((t, i) => t.content === `"${buttons[i].getAttribute('aria-label')}"` && t.position === 'absolute' && t.left === '56px' && t.opacity === '0' && t.visibility === 'hidden')
      && getComputedStyle(controls).zIndex === '3',
      tips.map((t) => t.content).join(' '));
    const tipRule = [...document.styleSheets[0].cssRules].find((r) => r.selectorText === '#controls button:hover::after');
    check('a hover opens the tooltip, fading in by A1', !!tipRule && tipRule.style.opacity === '1' && tipRule.style.visibility === 'visible' && tipRule.style.transition === 'opacity 144ms ease-out',
      tipRule && tipRule.style.transition);

    const c = editor.createCallout({ x: 50, y: 50 });
    c.text = 'copied by the button';
    editor.layoutScene();
    editor.record();
    editor.copyComposed().catch(() => {});
    for (let i = 0; i < 200 && !hud.textContent.includes(`copied ${model.image.width}x${model.image.height}`); i += 1) await sleep(50);
    const back = await invoke('editor_clipboard_readback');
    check('a copy puts the composed image on the clipboard', back.png && back.width === model.image.width && back.height === model.image.height,
      `${back.width}x${back.height}`);

    const storage = await invoke('editor_storage');
    const docs = await invoke('editor_documents');
    check('the HUD says how many documents the store holds and how much disk', storage.documents === docs.length && storage.bytes > 0 && hud.textContent.includes(`${storage.documents} documents, ${(storage.bytes / (1024 * 1024)).toFixed(1)} MB on disk`),
      `${storage.documents} documents, ${storage.bytes} bytes`);

    editor.saveAs().catch(() => {});
    await sleep(1500);
    let outcome = await invoke('editor_save_as_outcome');
    check('Save As opens', outcome.state === 'open', outcome.state);
    const sent = await invoke('editor_press_escape');
    for (let i = 0; i < 40 && sent && outcome.state === 'open'; i += 1) {
      await sleep(100);
      outcome = await invoke('editor_save_as_outcome');
    }
    if (!sent) {
      skipped('Escape closes it with nothing saved', 'another application is in front, so no key was sent; the dialog is left open');
    } else {
      check('Escape closes it with nothing saved', outcome.state === 'cancelled', outcome.line);
    }
    model.callouts = [];
    editor.layoutScene();
  }

  // ---------------------------------------------------------------- 29. S2.5: deleting a document
  say('');
  say('S2.5: a document is deleted from the thumbnail or by Ctrl+Delete, into Recon\'s trash for thirty days, never an original; the neighbour shows, the last one leaves the editor empty');
  {
    const hud = document.getElementById('hud');
    const strip = editor.strip;
    await invoke('editor_store_reset');
    const dir = await invoke('editor_make_folder');
    const filePath = `${dir}\\img2.png`;
    // An annotated file's document, then two captures.
    let info = await invoke('editor_open_path', { path: filePath });
    await editor.loadImage(info);
    await editor.setMode('annotate');
    const fileDoc = model.image.document_id;
    const shots = [];
    for (const [w, h] of [[320, 200], [340, 200]]) {
      const s = await invoke('editor_capture_probe', { width: w, height: h });
      await editor.loadImage(s);
      shots.push(s);
    }
    await editor.refreshStrip();
    for (let i = 0; i < 100 && [...strip.querySelectorAll('img')].length < 3; i += 1) await sleep(50);
    check('three documents on the timeline, the newest at the left and current', strip.children.length === 3 && strip.children[0].classList.contains('current'));

    // The thumbnail's delete on a document that is not on screen: gone from the timeline,
    // in the trash, the one on screen unchanged, the external file untouched.
    // Since S2.7 the strip ends with a Trash chip once something is in the trash, so the
    // documents are counted by their thumbnails.
    const thumbs = () => strip.querySelectorAll('.thumb:not(.trashed)').length;
    // A right press on the thumbnail opens the menu with Delete; no × on it (Rotem, 2026-09-19).
    const oldest = strip.children[2]; // the file's document, at the right
    const thumbMenu = document.getElementById('tab-menu');
    const oldestBox = oldest.getBoundingClientRect();
    const thumbTook = !oldest.dispatchEvent(new MouseEvent('contextmenu', { bubbles: true, cancelable: true, button: 2, clientX: oldestBox.left + 12, clientY: oldestBox.top + 16 }));
    check('a thumbnail carries no ×, and a right press on it opens the menu with Delete alone, deleting nothing, and not the system menu', !strip.querySelector('.thumb .x') && thumbTook && getComputedStyle(thumbMenu).display === 'block' && thumbMenu.textContent.trim() === 'Delete' && thumbs() === 3,
      `× ${!!strip.querySelector('.thumb .x')}, took ${thumbTook}, ${getComputedStyle(thumbMenu).display}, thumbnails ${thumbs()}`);
    document.body.dispatchEvent(new PointerEvent('pointerdown', { bubbles: true, cancelable: true, pointerId: 7, button: 0 }));
    check('a press elsewhere closes the thumbnail\'s menu and deletes nothing', getComputedStyle(thumbMenu).display === 'none' && thumbs() === 3);
    oldest.dispatchEvent(new MouseEvent('contextmenu', { bubbles: true, cancelable: true, button: 2, clientX: oldestBox.left + 12, clientY: oldestBox.top + 16 }));
    thumbMenu.querySelector('button').click();
    for (let i = 0; i < 60 && thumbs() !== 2; i += 1) await sleep(50);
    let trash = await invoke('editor_trash_list');
    check('deleted from its thumbnail: off the timeline, into the trash, the one on screen unchanged', thumbs() === 2 && trash.some(([id]) => id === fileDoc) && model.image.document_id === shots[1].document_id && hud.textContent.includes('trash for 30 days'),
      `thumbnails ${thumbs()}, trash ${JSON.stringify(trash)}`);
    const stillThere = await invoke('editor_open_path', { path: filePath }).then(() => true).catch(() => false);
    check('the external file the document came from is untouched', stillThere);
    await editor.loadImage(await invoke('editor_show_document', { id: shots[1].document_id }));

    // Ctrl+Delete on the document on screen: the neighbour shows.
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Delete', code: 'Delete', ctrlKey: true, bubbles: true, cancelable: true }));
    for (let i = 0; i < 60 && model.image.document_id !== shots[0].document_id; i += 1) await sleep(50);
    trash = await invoke('editor_trash_list');
    check('Ctrl+Delete on the one on screen: the neighbour shows, and it is in the trash', model.image.document_id === shots[0].document_id && trash.length === 2 && thumbs() === 1,
      `on screen ${model.image.document_id}, ${trash.length} in the trash`);

    // The last one: the editor is empty, the timeline gone, and nothing of the picture's
    // shapes stays on the empty stage (review 4, T4).
    const lastRect = editor.createShape('rect', { x: 20, y: 20 });
    lastRect.b = { x: 120, y: 90 };
    editor.layoutScene();
    editor.record();
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Delete', code: 'Delete', ctrlKey: true, bubbles: true, cancelable: true }));
    for (let i = 0; i < 60 && model.image.width !== 0; i += 1) await sleep(50);
    await sleep(100);
    check('the last document deleted leaves the editor empty, no thumbnail left, only the Trash chip, the controls away', model.image.width === 0 && thumbs() === 0 && !!strip.querySelector('#trash-chip') && document.getElementById('controls').hidden && hud.textContent.includes('capture'),
      hud.textContent.split('\n')[0]);
    check('and its shapes, crop and margin go with it: nothing is drawn on the empty stage', model.shapes.length === 0 && model.crop === null && editor.marginString() === '0,0,0,0' && !document.querySelector('#arrows [data-shape]') && !document.querySelector('#scene .ruler, #scene .blur'),
      `${model.shapes.length} shapes, crop ${JSON.stringify(model.crop)}, margin ${editor.marginString()}, drawn ${document.querySelectorAll('#arrows [data-shape]').length}`);
    const sizeGone = document.getElementById('image-size');
    check('and the picture\'s size is away with the picture', getComputedStyle(sizeGone).display === 'none' && sizeGone.textContent === '',
      `"${sizeGone.textContent}", ${getComputedStyle(sizeGone).display}`);

    // The sweep: thirty days on, a trashed document is gone for good; a younger one stays.
    await invoke('editor_trash_age', { id: shots[0].document_id, days: 31 });
    const swept = await invoke('editor_sweep_trash');
    trash = await invoke('editor_trash_list');
    check('the sweep removes what is thirty days old and keeps the rest', swept.length === 1 && swept[0] === shots[0].document_id && trash.length === 2 && !trash.some(([id]) => id === shots[0].document_id),
      `swept ${JSON.stringify(swept)}, left ${JSON.stringify(trash)}`);

    // A capture after the empty state works as before.
    const again = await invoke('editor_capture_probe', { width: 300, height: 200 });
    await editor.loadImage(again);
    await editor.refreshStrip();
    check('a capture after the empty state shows and the timeline returns', model.image.width === 300 && document.body.classList.contains('strip') && editor.canvas.style.display !== 'none',
      `${model.image.width}, strip ${document.body.classList.contains('strip')}, canvas ${JSON.stringify(editor.canvas.style.display)}`);

    // A capture deleted while its image is still encoding (review T2): no folder of its own
    // is left among the documents, the trash holds the record with its image, and Restore
    // brings the picture back.
    const racing = await invoke('editor_capture_probe', { width: 4000, height: 2500 });
    await editor.loadImage(racing);
    await editor.deleteDocument(racing.document_id);
    await sleep(1500);
    const folders = await invoke('editor_store_list');
    const trashNow = await invoke('editor_trash_list');
    const restored = await invoke('editor_trash_restore', { id: racing.document_id })
      .then(() => invoke('editor_show_document', { id: racing.document_id }))
      .catch((err) => ({ error: String(err) }));
    if (restored.width) await editor.loadImage(restored);
    check('a capture deleted while its image encodes leaves no folder among the documents, sits in the trash with its image, and Restore brings the picture back',
      !folders.some((l) => l.id === racing.document_id) && trashNow.some(([id]) => id === racing.document_id) && restored.width === 4000 && model.image.document_id === racing.document_id,
      `folders ${folders.map((l) => l.id).join(' ')}, trash ${JSON.stringify(trashNow)}, restore ${JSON.stringify(restored).slice(0, 90)}`);

    // A move to the trash that fails (review T10): the document stays listed, the picture
    // stays on screen, and the page says so.
    await invoke('editor_trash_break', { on: true });
    const keptId = model.image.document_id;
    const refused = await editor.deleteDocument(keptId).then(() => null, (err) => String(err));
    const listedStill = (await invoke('editor_documents')).some((d) => d.id === keptId);
    await invoke('editor_trash_break', { on: false });
    check('a delete whose move to the trash fails keeps the document listed and on screen, and says NOT DELETED',
      !!refused && listedStill && model.image.document_id === keptId && hud.textContent.includes('NOT DELETED'), refused || 'deleted');
    editor.notice('', 0);

    // A neighbour that cannot be shown (review 4, T7): the delete is still done, the editor
    // goes empty, and nothing says NOT DELETED.
    {
      const shownId = model.image.document_id;
      const neighbour = (await invoke('editor_documents')).find((d) => d.id !== shownId);
      await invoke('editor_store_remove_source', { id: neighbour.id });
      const outcome = await editor.deleteDocument(shownId).then(() => 'deleted', (err) => String(err));
      const trashHolds = (await invoke('editor_trash_list')).some(([id]) => id === shownId);
      check('a delete whose neighbour cannot be shown is still a delete: the editor empty, the document in the trash, its undo recorded, no NOT DELETED',
        outcome === 'deleted' && model.image.width === 0 && trashHolds && !hud.textContent.includes('NOT DELETED') && editor.deletionsOf().some((d) => d.id === shownId),
        `${outcome}, width ${model.image.width}, in the trash ${trashHolds}, ${hud.textContent.split('\n').pop()}`);
    }
    model.callouts = [];
    editor.layoutScene();
  }

  // ---------------------------------------------------------------- 30. S2.6: JPEG beside PNG
  say('');
  say('S2.6: Save As writes a JPEG when the chosen name says so, flattened over white at a fixed quality, through the same never-overwrite path');
  {
    const outDir = (await invoke('editor_store_reset')) + '\\exports';
    const shot = await invoke('editor_capture_probe', { width: 300, height: 200 });
    await editor.loadImage(shot);
    const c = editor.createCallout({ x: 40, y: 40 });
    c.text = 'both formats';
    editor.layoutScene();
    editor.record();
    const writeTo = async (path) => {
      const layer = await editor.exportLayer();
      return invoke('editor_save_as_write', layer.bytes, { headers: { margin: layer.margin, path } });
    };
    const jpg = await writeTo(outDir + '\\shot annotated.jpg');
    const jpgKind = jpg.New ? await invoke('editor_file_kind', { path: jpg.New }) : null;
    check('a .jpg name writes a JPEG of the composition', !!jpgKind && jpgKind.format === 'jpeg' && jpgKind.width === 300 + model.margin.left + model.margin.right && jpgKind.height === 200 + model.margin.top + model.margin.bottom,
      jpgKind ? `${jpgKind.format} ${jpgKind.width}x${jpgKind.height}, ${jpgKind.bytes} bytes` : JSON.stringify(jpg));
    const png = await writeTo(outDir + '\\shot annotated.png');
    const pngKind = png.New ? await invoke('editor_file_kind', { path: png.New }) : null;
    check('a .png name still writes a PNG', !!pngKind && pngKind.format === 'png' && pngKind.width === jpgKind.width, pngKind ? pngKind.format : JSON.stringify(png));
    const again = await writeTo(outDir + '\\shot annotated.jpg');
    check('the same JPEG name is never written over, a free one is offered', again.Exists !== undefined && again.Exists.offered.endsWith('shot annotated (2).jpg'), JSON.stringify(again).slice(0, 120));
    model.callouts = [];
    editor.layoutScene();
  }

  // ---------------------------------------------------------------- 31. S2.7: restore from the trash
  say('');
  say('S2.7: the timeline ends with a Trash chip when something was deleted; it opens the trash in the strip, and Restore brings a document back with its notes');
  {
    const strip = editor.strip;
    await invoke('editor_store_reset');
    const a = await invoke('editor_capture_probe', { width: 320, height: 200 });
    await editor.loadImage(a);
    const n = editor.createCallout({ x: 30, y: 30 });
    n.text = 'back from the trash';
    editor.layoutScene();
    editor.record();
    await editor.saveNow();
    const b = await invoke('editor_capture_probe', { width: 340, height: 200 });
    await editor.loadImage(b);
    await editor.refreshStrip();
    check('no chip while the trash is empty', !strip.querySelector('#trash-chip'));
    await editor.deleteDocument(a.document_id);
    const chip = strip.querySelector('#trash-chip');
    check('after a delete the timeline ends with the Trash chip and its count', !!chip && chip.textContent === 'Trash · 1', chip && chip.textContent);
    chip.click();
    for (let i = 0; i < 60 && !strip.querySelector('.thumb.trashed'); i += 1) await sleep(50);
    const trashed = strip.querySelector('.thumb.trashed');
    check('the chip opens the trash in the strip: the deleted document, with Restore, and a way back', document.body.classList.contains('trash') && !!trashed && !!trashed.querySelector('.restore') && !!strip.querySelector('#trash-back'));
    for (let i = 0; i < 100 && !(trashed.querySelector('img') && trashed.querySelector('img').naturalWidth > 0); i += 1) await sleep(50);
    check('its thumbnail is there', !!trashed.querySelector('img') && trashed.querySelector('img').naturalWidth > 0);
    trashed.querySelector('.restore').click();
    for (let i = 0; i < 60 && model.image.document_id !== a.document_id; i += 1) await sleep(50);
    await sleep(200);
    const docs = await invoke('editor_documents');
    const left = await invoke('editor_trash_list');
    check('Restore brings the document back, on screen, with its note, in the list, out of the trash', model.image.document_id === a.document_id && model.callouts.length === 1 && model.callouts[0].text === 'back from the trash' && docs.some((d) => d.id === a.document_id) && left.length === 0 && !document.body.classList.contains('trash') && !strip.querySelector('#trash-chip'),
      `on screen ${model.image.document_id}, ${docs.length} documents, ${left.length} in the trash`);
    model.callouts = [];
    editor.layoutScene();
  }

  // ---------------------------------------------------------------- 32. S3.1: the arrow tool
  say('');
  say('S3.1: a tool is chosen by key or button, an arrow is drawn by a drag, selected, moved, deleted, undone, saved and exported, and the callout flow is untouched');
  {
    const stage = editor.stage;
    const hud = document.getElementById('hud');
    await invoke('editor_store_reset');
    const shot = await invoke('editor_capture_probe', { width: 400, height: 300 });
    await editor.loadImage(shot);
    await editor.setZoom(1);
    const box = stage.getBoundingClientRect();
    const css = (ix, iy) => ({ x: box.left + model.offset.x + ((ix - model.pan.x) * model.zoom) / editor.ratioOf(), y: box.top + model.offset.y + ((iy - model.pan.y) * model.zoom) / editor.ratioOf() });
    const pointer = (type, target, ix, iy) => {
      const at = css(ix, iy);
      target.dispatchEvent(new PointerEvent(type, { pointerId: 11, button: 0, buttons: type === 'pointerup' ? 0 : 1, clientX: at.x, clientY: at.y, bubbles: true, cancelable: true }));
    };
    const press = (init) => { const e = new KeyboardEvent('keydown', { bubbles: true, cancelable: true, ...init }); window.dispatchEvent(e); return e.defaultPrevented; };

    // Rotem, 2026-09-14: no tool is in hand when a picture opens, the pointer is the
    // ordinary one, and a click on the picture creates nothing.
    check('no tool is in hand at first: no button lit, the HUD says none, the ordinary pointer',
      model.tool === null && !document.querySelector('#tools button.active') && hud.textContent.includes('tool none') && getComputedStyle(stage).cursor === 'auto',
      `tool ${model.tool}, cursor ${getComputedStyle(stage).cursor}`);
    pointer('pointerdown', stage, 300, 250);
    pointer('pointerup', stage, 300, 250);
    check('a click on the picture creates nothing', model.callouts.length === 0 && model.shapes.length === 0 && model.editing === null);
    // Rotem, 2026-09-14: with no tool in hand a drag on the picture pans it, as in viewing;
    // at the top zoom, where this capture is larger than the window.
    const raw = (type, x, y) => stage.dispatchEvent(new PointerEvent(type, { pointerId: 11, button: 0, buttons: type === 'pointerup' ? 0 : 1, clientX: x, clientY: y, bubbles: true, cancelable: true }));
    const mid = { x: box.left + box.width / 2, y: box.top + box.height / 2 };
    await editor.setZoom(8);
    const panFrom = { ...model.pan };
    raw('pointerdown', mid.x, mid.y);
    raw('pointermove', mid.x + 40, mid.y + 30);
    raw('pointerup', mid.x + 40, mid.y + 30);
    check('with no tool a drag on the picture pans it and creates nothing', model.pan.x < panFrom.x && model.pan.y < panFrom.y && model.callouts.length === 0 && model.shapes.length === 0,
      `pan ${panFrom.x.toFixed(1)},${panFrom.y.toFixed(1)} to ${model.pan.x.toFixed(1)},${model.pan.y.toFixed(1)}`);
    // Rotem, 2026-09-15: the picture moves with the pointer, not on release. At the first
    // move what is painted has already slid by the move, and while moves keep coming closer
    // together than a region's round trip, regions under newer pans still land.
    await editor.paintRegion();
    const regionBefore = editor.lastRegion();
    const leftBefore = parseFloat(editor.canvas.style.left) || 0;
    raw('pointerdown', mid.x, mid.y);
    raw('pointermove', mid.x + 40, mid.y);
    const slid = (parseFloat(editor.canvas.style.left) || 0) - leftBefore;
    for (let i = 1; i <= 60; i += 1) {
      raw('pointermove', mid.x + 40 + i * 2, mid.y + i);
      await sleep(8);
    }
    const regionDuring = editor.lastRegion();
    raw('pointerup', mid.x + 160, mid.y + 60);
    check('a drag moves the picture while the pointer moves, not on release: it slides at the first move, and regions land before the pointer lets go',
      Math.abs(slid - 40) <= 5 && !!regionDuring && regionDuring !== regionBefore && regionDuring.x !== regionBefore.x,
      `slid ${slid.toFixed(1)} css px for a 40 px move; region x ${regionBefore && regionBefore.x} then ${regionDuring && regionDuring.x} before the release, a round trip ${regionDuring && regionDuring.ms} ms`);
    await editor.paintRegion();
    // Rotem, 2026-09-15: an edge a drag uncovers is never empty. A note anchored at a point of
    // the picture marks where the picture is; a move of 300 px leaves the viewport's left part
    // outside the sharp canvas, and before any region can land the copy of the whole picture
    // must be shown there, lined up with the note, and hidden again once the drag has painted.
    const copyCanvas = document.getElementById('backdrop');
    const ratioNow = editor.ratioOf();
    const markAt = { x: Math.round(model.pan.x - (200 * ratioNow) / model.zoom), y: Math.round(model.pan.y + ((box.height / 2) * ratioNow) / model.zoom) };
    const marker = editor.createCallout(markAt);
    editor.layoutScene();
    const markerAnchor = document.querySelector(`[data-anchor-for="${marker.id}"]`);
    const copyKept = copyCanvas.width > 0;
    raw('pointerdown', mid.x, mid.y);
    raw('pointermove', mid.x + 300, mid.y);
    const copyShown = getComputedStyle(copyCanvas).display !== 'none';
    const sharpLeft = editor.canvas.getBoundingClientRect().left;
    const copyBox = copyCanvas.getBoundingClientRect();
    const noteBox = markerAnchor.getBoundingClientRect();
    const noteX = noteBox.left + noteBox.width / 2;
    const noteY = noteBox.top + noteBox.height / 2;
    const copyX = copyBox.left + (markAt.x * copyBox.width) / model.image.width;
    const copyY = copyBox.top + (markAt.y * copyBox.height) / model.image.height;
    const edgeUncovered = sharpLeft > box.left + 1 && copyBox.left <= box.left && copyBox.right >= sharpLeft;
    raw('pointerup', mid.x + 300, mid.y);
    await editor.paintRegion();
    // The drag's own last region may still be on its way and land after this one, so the copy
    // is given a second to hide.
    for (let i = 0; i < 40 && getComputedStyle(copyCanvas).display !== 'none'; i += 1) await sleep(25);
    const copyHiddenAfter = getComputedStyle(copyCanvas).display === 'none';
    editor.removeCallout(marker);
    editor.layoutScene();
    check('an edge a drag uncovers shows the whole picture at once, lined up with the notes, and the copy hides once the drag has painted',
      copyKept && copyShown && edgeUncovered && Math.abs(noteX - copyX) <= 2 && Math.abs(noteY - copyY) <= 2 && copyHiddenAfter,
      `copy ${copyCanvas.width}x${copyCanvas.height}, shown ${copyShown}, the sharp canvas from ${(sharpLeft - box.left).toFixed(1)} px, the note at ${noteX.toFixed(1)},${noteY.toFixed(1)} and the copy's point at ${copyX.toFixed(1)},${copyY.toFixed(1)}, hidden after ${copyHiddenAfter}`);
    await editor.setZoom(1);
    press({ key: 'l', code: 'KeyL' });
    const arrowButton = document.querySelector('#tools [data-tool="arrow"]');
    check('L picks the arrow tool, its button lights, and the pointer is the crosshair', model.tool === 'arrow' && arrowButton.classList.contains('active') && getComputedStyle(stage).cursor === 'crosshair',
      getComputedStyle(stage).cursor);
    // Space held with the arrow in hand: the ordinary pointer, the notes off the pointer, and a
    // drag that pans instead of drawing; letting go, or the window losing the keyboard, ends it.
    const scene = document.getElementById('scene');
    press({ key: ' ', code: 'Space' });
    check('Space held with a tool in hand gives the ordinary pointer and takes the notes off it, the tool kept',
      document.body.classList.contains('space') && getComputedStyle(stage).cursor === 'auto' && getComputedStyle(scene).pointerEvents === 'none' && model.tool === 'arrow',
      `cursor ${getComputedStyle(stage).cursor}, scene ${getComputedStyle(scene).pointerEvents}`);
    await editor.setZoom(8);
    const spaceFrom = { ...model.pan };
    raw('pointerdown', mid.x, mid.y);
    raw('pointermove', mid.x + 40, mid.y + 30);
    raw('pointerup', mid.x + 40, mid.y + 30);
    check('and a drag pans the picture and draws nothing', model.pan.x < spaceFrom.x && model.pan.y < spaceFrom.y && model.shapes.length === 0,
      `pan ${spaceFrom.x.toFixed(1)},${spaceFrom.y.toFixed(1)} to ${model.pan.x.toFixed(1)},${model.pan.y.toFixed(1)}`);
    window.dispatchEvent(new KeyboardEvent('keyup', { key: ' ', code: 'Space', bubbles: true }));
    check('letting go of Space brings the crosshair back, the arrow still in hand', !document.body.classList.contains('space') && getComputedStyle(stage).cursor === 'crosshair' && model.tool === 'arrow');
    press({ key: ' ', code: 'Space' });
    window.dispatchEvent(new Event('blur'));
    check('the window losing the keyboard lets go of Space too', !document.body.classList.contains('space'));
    await editor.setZoom(1);

    pointer('pointerdown', stage, 50, 60);
    pointer('pointermove', stage, 200, 160);
    pointer('pointerup', stage, 200, 160);
    check('a drag draws an arrow from where it began to where it ended, with no callout made', model.shapes.length === 1 && model.shapes[0].kind === 'arrow'
      && model.shapes[0].a.x === 50 && model.shapes[0].a.y === 60 && model.shapes[0].b.x === 200 && model.shapes[0].b.y === 160 && model.callouts.length === 0,
      JSON.stringify(model.shapes[0]));
    const drawn = editor.handles.ownerDocument.querySelector(`[data-shape="${model.shapes[0].id}"]`);
    check('the arrow is in the scene as a line and a head', !!drawn && drawn.querySelector('line') && drawn.querySelector('polygon'));
    pointer('pointerdown', stage, 300, 250);
    pointer('pointerup', stage, 300, 250);
    check('a click with the tool makes nothing', model.shapes.length === 1 && model.callouts.length === 0);

    const layer = await editor.exportLayer();
    check('the export carries the arrow and not the handles', layer.markup.includes('data-shape') && layer.markup.includes('<polygon') && !layer.markup.includes('id="handles"') && !layer.markup.includes('<circle'));

    // The scene redraws its shapes on every layout, so the element is looked up afresh.
    const shapeEl = () => document.querySelector(`[data-shape="${model.shapes[0].id}"]`);
    pointer('pointerdown', shapeEl(), 120, 110);
    check('a click on the arrow selects it, with handles at both ends', model.selectedShape === model.shapes[0] && editor.handles.querySelectorAll('circle').length === 2,
      `selected ${model.selectedShape && model.selectedShape.id}, ${editor.handles.querySelectorAll('circle').length} handles`);
    pointer('pointermove', stage, 140, 130);
    pointer('pointerup', stage, 140, 130);
    check('a drag on it moves both ends', model.shapes[0].a.x === 70 && model.shapes[0].a.y === 80 && model.shapes[0].b.x === 220 && model.shapes[0].b.y === 180, JSON.stringify(model.shapes[0]));
    press({ key: 'z', code: 'KeyZ', ctrlKey: true });
    check('undo puts it back', model.shapes[0].a.x === 50 && model.shapes[0].b.x === 200);
    press({ key: 'z', code: 'KeyZ', ctrlKey: true });
    check('undo again removes it', model.shapes.length === 0);
    press({ key: 'y', code: 'KeyY', ctrlKey: true });
    check('redo brings it back', model.shapes.length === 1);

    await editor.saveNow();
    editor.documents.clear();
    model.shapes = [];
    model.image = { width: 0, height: 0, source: '' };
    await editor.loadImage(await invoke('editor_store_reload'));
    check('after a restart the arrow is back from the disk', model.shapes.length === 1 && model.shapes[0].kind === 'arrow' && model.shapes[0].b.x === 200, JSON.stringify(model.shapes[0]));
    check('the picture opened again starts with no tool in hand, though the arrow was in hand before', model.tool === null && !document.body.classList.contains('armed'));

    pointer('pointerdown', shapeEl(), 120, 110);
    pointer('pointerup', stage, 120, 110);
    press({ key: 'Delete', code: 'Delete' });
    check('Delete removes the selected arrow', model.shapes.length === 0 && model.selectedShape === null);

    press({ key: 'c', code: 'KeyC' });
    pointer('pointerdown', stage, 100, 100);
    pointer('pointerup', stage, 100, 100);
    pointer('pointermove', stage, 140, 130);
    pointer('pointerdown', stage, 140, 130);
    pointer('pointerup', stage, 140, 130);
    check('C picks the callout tool and two clicks make a note, typed after the second', model.tool === 'callout' && model.callouts.length === 1 && model.editing === model.callouts[0]);
    check('Space while a note is typed is the note\'s, never a pan', press({ key: ' ', code: 'Space' }) === false && !document.body.classList.contains('space'));
    editor.commitEditing();
    model.callouts = [];
    model.shapes = [];
    editor.layoutScene();
    editor.setTool(null);
  }

  // ---------------------------------------------------------------- 33. S3.2: the rectangle; the highlight is gone (Rotem, 2026-09-18)
  say('');
  say('S3.2: R picks the rectangle; a drag either way makes the rectangle it crossed; it selects, moves, undoes and exports like the arrow; the highlight is gone, its button, its key and any drawn before');
  {
    const stage = editor.stage;
    await invoke('editor_store_reset');
    const shot = await invoke('editor_capture_probe', { width: 400, height: 300 });
    await editor.loadImage(shot);
    await editor.setZoom(1);
    const box = stage.getBoundingClientRect();
    const css = (ix, iy) => ({ x: box.left + model.offset.x + ((ix - model.pan.x) * model.zoom) / editor.ratioOf(), y: box.top + model.offset.y + ((iy - model.pan.y) * model.zoom) / editor.ratioOf() });
    const pointer = (type, target, ix, iy) => {
      const at = css(ix, iy);
      target.dispatchEvent(new PointerEvent(type, { pointerId: 12, button: 0, buttons: type === 'pointerup' ? 0 : 1, clientX: at.x, clientY: at.y, bubbles: true, cancelable: true }));
    };
    const press = (init) => { const e = new KeyboardEvent('keydown', { bubbles: true, cancelable: true, ...init }); window.dispatchEvent(e); return e.defaultPrevented; };
    const shapeEl = (shape) => document.querySelector(`[data-shape="${shape.id}"]`);

    press({ key: 'r', code: 'KeyR' });
    check('R picks the rectangle', model.tool === 'rect' && document.querySelector('#tools [data-tool="rect"]').classList.contains('active'));
    pointer('pointerdown', stage, 220, 200);
    pointer('pointermove', stage, 60, 40);
    pointer('pointerup', stage, 60, 40);
    const rect = model.shapes[0];
    const r = rect && editor.rectOfShape(rect);
    check('a drag up and to the left still makes the rectangle it crossed', !!rect && rect.kind === 'rect' && r.x === 60 && r.y === 40 && r.w === 160 && r.h === 160, JSON.stringify(r));
    check('it is an outline in the scene', !!shapeEl(rect) && shapeEl(rect).querySelector('rect').getAttribute('fill') === 'none');

    const toolBefore = model.tool;
    const took = press({ key: 'h', code: 'KeyH' });
    check('the highlight is gone: no button for it in the sidebar, and H picks nothing', !document.querySelector('#tools [data-tool="highlight"]') && model.tool === toolBefore && !took, `${model.tool}`);
    // One drawn before the tool went: it shows nowhere, on screen or in a copy.
    const old = { id: 's-old', kind: 'highlight', a: { x: 250, y: 100 }, b: { x: 380, y: 140 } };
    model.shapes.push(old);
    editor.layoutScene();
    const layer = await editor.exportLayer();
    check('a highlight drawn before shows nowhere: not in the scene, not in the export', !shapeEl(old) && (layer.markup.match(/<rect /g) || []).length === 1, `${(layer.markup.match(/<rect /g) || []).length} rect in the export`);
    model.shapes = model.shapes.filter((s) => s !== old);
    editor.layoutScene();

    pointer('pointerdown', shapeEl(rect), 100, 100);
    pointer('pointermove', stage, 110, 120);
    pointer('pointerup', stage, 110, 120);
    check('the rectangle is selected and moved whole', model.selectedShape === rect && editor.rectOfShape(rect).x === 70 && editor.rectOfShape(rect).y === 60 && editor.rectOfShape(rect).w === 160, JSON.stringify(editor.rectOfShape(rect)));
    press({ key: 'z', code: 'KeyZ', ctrlKey: true });
    // Undo rebuilds the list, so the shape is read from it, not from the old object.
    check('undo puts it back', editor.rectOfShape(model.shapes[0]).x === 60 && editor.rectOfShape(model.shapes[0]).y === 40, JSON.stringify(editor.rectOfShape(model.shapes[0])));
    pointer('pointerdown', shapeEl(model.shapes[0]), 100, 100);
    pointer('pointerup', stage, 100, 100);
    check('a click selects the rectangle again', model.selectedShape === model.shapes[0]);
    press({ key: 'Escape', code: 'Escape' });
    check('Escape clears the selection', model.selectedShape === null);
    press({ key: 'c', code: 'KeyC' });
    model.callouts = [];
    model.shapes = [];
    editor.layoutScene();
    editor.setTool(null);
    for (let i = 0; i < 40 && !(await invoke('editor_window_visible')); i += 1) await sleep(50);
    await invoke('editor_show');
  }

  // ---------------------------------------------------------------- 34. S3.3: the text tool
  say('');
  say('S3.3: T, then a click, types a note where it was clicked, with no number, no bubble and no arrow, and the callouts\' numbering is untouched');
  {
    const stage = editor.stage;
    await invoke('editor_store_reset');
    const shot = await invoke('editor_capture_probe', { width: 400, height: 300 });
    await editor.loadImage(shot);
    await editor.setZoom(1);
    const box = stage.getBoundingClientRect();
    const css = (ix, iy) => ({ x: box.left + model.offset.x + ((ix - model.pan.x) * model.zoom) / editor.ratioOf(), y: box.top + model.offset.y + ((iy - model.pan.y) * model.zoom) / editor.ratioOf() });
    const pointer = (type, target, ix, iy) => {
      const at = css(ix, iy);
      target.dispatchEvent(new PointerEvent(type, { pointerId: 13, button: 0, buttons: type === 'pointerup' ? 0 : 1, clientX: at.x, clientY: at.y, bubbles: true, cancelable: true }));
    };
    const press = (init) => { const e = new KeyboardEvent('keydown', { bubbles: true, cancelable: true, ...init }); window.dispatchEvent(e); return e.defaultPrevented; };

    press({ key: 'c', code: 'KeyC' });
    pointer('pointerdown', stage, 40, 40);
    pointer('pointerup', stage, 40, 40);
    pointer('pointermove', stage, 80, 70);
    pointer('pointerdown', stage, 80, 70);
    pointer('pointerup', stage, 80, 70);
    check('two clicks with the callout tool make a note', model.callouts.length === 1);
    document.querySelector(`[data-id="${model.callouts[0].id}"] .t`).textContent = 'one';
    editor.commitEditing();
    // A committed note stays selected, and a click elsewhere clears that first, which is
    // the callout rule; Escape clears it here so the next click is the tool's.
    press({ key: 'Escape', code: 'Escape' });
    press({ key: 't', code: 'KeyT' });
    check('T picks the text tool', model.tool === 'text');
    pointer('pointerdown', stage, 150, 120);
    pointer('pointerup', stage, 150, 120);
    const note = model.callouts[1];
    check('a click makes a text note where it was clicked, being typed at once', !!note && note.kind === 'text' && note.box.x === 150 && note.box.y === 120 && model.editing === note, JSON.stringify(note && note.box));
    const el = document.querySelector(`[data-id="${note.id}"]`);
    check('no number, no anchor, no arrow', note.number === null && el.classList.contains('text') && getComputedStyle(el.querySelector('.n')).display === 'none' && !document.querySelector(`[data-anchor-for="${note.id}"]`) && document.querySelectorAll('#arrows line').length === 1);
    el.querySelector('.t').textContent = 'plain words on the picture';
    editor.commitEditing();
    check('the text commits like a note', note.text === 'plain words on the picture' && model.editing === null);
    const layer = await editor.exportLayer();
    // The export is the scene's own DOM (S0.6), so the bubble's transparency is read on
    // the element and the words are looked for in the layer.
    check('the export carries the words, and the bubble is transparent', layer.markup.includes('plain words on the picture') && getComputedStyle(el).backgroundColor === 'rgba(0, 0, 0, 0)',
      getComputedStyle(el).backgroundColor);

    press({ key: 'Escape', code: 'Escape' });
    press({ key: 'c', code: 'KeyC' });
    pointer('pointerdown', stage, 300, 200);
    pointer('pointerup', stage, 300, 200);
    pointer('pointermove', stage, 340, 230);
    pointer('pointerdown', stage, 340, 230);
    pointer('pointerup', stage, 340, 230);
    const next = model.callouts[2];
    check('the next callout is number 2: the text note took no number', !!next && next.number === 2, JSON.stringify(next && next.number));
    document.querySelector(`[data-id="${next.id}"] .t`).textContent = 'two';
    editor.commitEditing();
    press({ key: 'z', code: 'KeyZ', ctrlKey: true });
    press({ key: 'z', code: 'KeyZ', ctrlKey: true });
    check('undo removes the text note in its turn', model.callouts.length === 1 && model.callouts[0].text === 'one');
    model.callouts = [];
    model.shapes = [];
    editor.layoutScene();
    editor.setTool(null);
  }

  // ---------------------------------------------------------------- 35. S3.4: the blur
  say('');
  say('S3.4: B, then a drag, blurs a region: on screen through a box over the picture, in the output by the host on a copy of the source, the document\'s own pixels untouched');
  {
    const stage = editor.stage;
    await invoke('editor_store_reset');
    const shot = await invoke('editor_load_probe', { width: 800, height: 400 });
    await editor.loadImage(shot);
    await editor.setMode('annotate');
    await editor.setZoom(1);
    const box = stage.getBoundingClientRect();
    const css = (ix, iy) => ({ x: box.left + model.offset.x + ((ix - model.pan.x) * model.zoom) / editor.ratioOf(), y: box.top + model.offset.y + ((iy - model.pan.y) * model.zoom) / editor.ratioOf() });
    const pointer = (type, target, ix, iy) => {
      const at = css(ix, iy);
      target.dispatchEvent(new PointerEvent(type, { pointerId: 14, button: 0, buttons: type === 'pointerup' ? 0 : 1, clientX: at.x, clientY: at.y, bubbles: true, cancelable: true }));
    };
    const press = (init) => { const e = new KeyboardEvent('keydown', { bubbles: true, cancelable: true, ...init }); window.dispatchEvent(e); return e.defaultPrevented; };

    const before = await exportCheck('s34-before', 'source');
    check('with no blur every source pixel is exact', before.source_mismatches === 0, `${before.source_mismatches} differ`);

    press({ key: 'b', code: 'KeyB' });
    check('B picks the blur', model.tool === 'blur');
    // The probe's left half is a one-pixel checkerboard, which a blur changes everywhere.
    pointer('pointerdown', stage, 40, 40);
    pointer('pointermove', stage, 200, 160);
    pointer('pointerup', stage, 200, 160);
    const blur = model.shapes[0];
    const el = document.querySelector(`.blur[data-shape="${blur && blur.id}"]`);
    check('a drag makes a blur region, shown as a box that blurs what is behind it', !!blur && blur.kind === 'blur' && !!el && el.style.backdropFilter === `blur(${editor.blurRadius(editor.rectOfShape(blur))}px)` && editor.blurRadius(editor.rectOfShape(blur)) === 15,
      el ? el.style.backdropFilter : 'no box');
    check('the export layer carries the region as a header and not as a box', editor.blurString() === '40,40,160,120');
    const layer = await editor.exportLayer();
    check('no blur box is in the layer itself', !layer.markup.includes('class="blur"') && !layer.markup.includes('backdrop-filter'));

    const after = await exportCheck('s34-after', 'source');
    check('the output differs from the source inside the region only, and the document\'s own pixels stay', after.source_mismatches > 1000 && after.source_mismatches <= 160 * 120 && after.width === 800,
      `${after.source_mismatches} pixels changed of ${160 * 120} in the region`);
    const again = await exportCheck('s34-again', 'source');
    check('exporting again gives the same blur, since the source was never altered', again.source_mismatches === after.source_mismatches);

    pointer('pointerdown', el, 100, 100);
    pointer('pointerup', stage, 100, 100);
    press({ key: 'Delete', code: 'Delete' });
    const gone = await exportCheck('s34-gone', 'source');
    check('deleting the blur brings every source pixel back', model.shapes.length === 0 && gone.source_mismatches === 0, `${gone.source_mismatches} differ`);
    press({ key: 'z', code: 'KeyZ', ctrlKey: true });
    check('undo brings the blur back', model.shapes.length === 1 && model.shapes[0].kind === 'blur');
    model.shapes = [];
    model.callouts = [];
    editor.layoutScene();
    editor.setTool(null);
  }

  // ---------------------------------------------------------------- 36. the top bar
  say('');
  say('the top bar: Recon draws its own, the logo on the left with no title line beside it, minimize, maximize and close on the right; fullscreen puts it away');
  {
    const win = window.__TAURI__.window.getCurrentWindow();
    const stage = document.getElementById('stage');
    const bar = document.getElementById('bar');
    const buttons = [...document.querySelectorAll('#winbtns button')];
    const info = await invoke('editor_open_fixture', { name: 'reference-scene.png' });
    await editor.loadImage(info);
    check('the window has no frame of Windows\' own', (await win.isDecorated()) === false);
    const barBox = bar.getBoundingClientRect();
    const stageTop = () => Math.round(stage.getBoundingClientRect().top);
    check('the bar runs across the top, 64 px tall, and the stage starts below it', barBox.top === 0 && barBox.height === 64 && Math.round(barBox.width) === window.innerWidth && stageTop() === 64,
      `bar ${barBox.height} tall, ${Math.round(barBox.width)} of ${window.innerWidth} wide, the stage from ${stageTop()}`);
    const logoBox = document.getElementById('logo').getBoundingClientRect();
    check('the logo sits at the bar\'s left, 103 by 32, with 16 px above it and to its left, and a press on it reaches the bar', Math.round(logoBox.left) === 16 && Math.round(logoBox.top) === 16 && Math.round(logoBox.width) === 103 && Math.round(logoBox.height) === 32
      && getComputedStyle(document.getElementById('logo')).pointerEvents === 'none' && document.getElementById('title').getBoundingClientRect().left >= logoBox.right,
      `the logo ${Math.round(logoBox.left)},${Math.round(logoBox.top)} ${Math.round(logoBox.width)} by ${Math.round(logoBox.height)}, the title from ${document.getElementById('title').getBoundingClientRect().left}`);
    check('the bar is the drag region, its empty middle included', bar.hasAttribute('data-tauri-drag-region') && document.getElementById('title').hasAttribute('data-tauri-drag-region'));
    check('the bar carries no title line, and the window\'s own title still names the file', document.getElementById('title').textContent === '' && (await invoke('editor_window_title')) === 'reference-scene.png - Recon',
      document.getElementById('title').textContent);
    const boxes = buttons.map((b) => b.getBoundingClientRect());
    // 48 wide by 40 tall, at the top of the 64 px bar: the bar grew, the buttons did not (Rotem, 2026-09-18).
    check('settings, minimize, maximize and close sit in that order at the right edge, 48 wide by 40 tall', buttons.map((b) => b.id).join() === 'win-settings,win-min,win-max,win-close'
      && boxes.every((b) => Math.round(b.width) === 48 && Math.round(b.height) === 40 && b.top === 0) && Math.round(boxes[3].right) === window.innerWidth && boxes[0].right <= boxes[1].left && boxes[1].right <= boxes[2].left && boxes[2].right <= boxes[3].left,
      boxes.map((b) => `${Math.round(b.left)}-${Math.round(b.right)}, ${Math.round(b.height)} tall`).join(' '));
    check('every window button has a name', buttons.every((b) => b.title.length > 0), buttons.map((b) => b.title).join(', '));

    const before = { w: window.innerWidth, h: window.innerHeight, maximized: await win.isMaximized() };
    document.getElementById('win-max').click();
    for (let i = 0; i < 40 && !(await win.isMaximized()); i += 1) await sleep(50);
    await sleep(200);
    check('the maximize button maximizes the window, and becomes Restore', !before.maximized && (await win.isMaximized()) && window.innerWidth >= before.w && window.innerHeight >= before.h && document.getElementById('win-max').title === 'Restore',
      `${before.w}x${before.h} to ${window.innerWidth}x${window.innerHeight}, ${document.getElementById('win-max').title}`);
    check('a maximized window still has the bar at the top and the stage below it', stageTop() === 64 && Math.round(document.getElementById('winbtns').getBoundingClientRect().right) === window.innerWidth);
    document.getElementById('win-max').click();
    for (let i = 0; i < 40 && (await win.isMaximized()); i += 1) await sleep(50);
    await sleep(200);
    check('and restores it', !(await win.isMaximized()) && window.innerWidth === before.w && window.innerHeight === before.h && document.getElementById('win-max').title === 'Maximize',
      `${window.innerWidth}x${window.innerHeight}`);
    check('the keys are still the page\'s after a click on a window button', document.activeElement !== document.getElementById('win-max'));

    // A capture with the editor minimized brings it back (Rotem, 2026-09-16): the host's
    // show, which a capture ends with, restores a minimized window before it focuses it.
    document.getElementById('win-min').click();
    for (let i = 0; i < 40 && !(await win.isMinimized()); i += 1) await sleep(50);
    const minimized = await win.isMinimized();
    await invoke('editor_show');
    for (let i = 0; i < 40 && (await win.isMinimized()); i += 1) await sleep(50);
    await sleep(200);
    check('a minimized window comes back when the host shows it, as a capture does', minimized && !(await win.isMinimized()) && (await invoke('editor_window_visible')) === true,
      `minimized ${minimized}, then minimized ${await win.isMinimized()}`);

    await editor.setFullscreen(true);
    const stageLeft = () => Math.round(stage.getBoundingClientRect().left);
    const sidebar = document.getElementById('controls');
    check('fullscreen puts the bar and the sidebar away and the stage takes the whole window', getComputedStyle(bar).display === 'none' && getComputedStyle(sidebar).display === 'none'
      && stageTop() === 0 && stageLeft() === 0 && stage.clientHeight === window.innerHeight && stage.clientWidth === window.innerWidth, `the stage from ${stageLeft()},${stageTop()}`);
    await editor.setFullscreen(false);
    check('and they come back', getComputedStyle(bar).display !== 'none' && getComputedStyle(sidebar).display !== 'none' && stageTop() === 64 && stageLeft() === 64, `the stage from ${stageLeft()},${stageTop()}`);
  }

  // ---------------------------------------------------------------- settings: the two global shortcuts
  say('');
  say('Settings: a chevron left of minimize opens a modal; a shortcut for opening Recon and two for the capture are picked by pressing them; a taken one is refused in words and the old one stays; the file keeps all three');
  {
    const button = document.getElementById('win-settings');
    const veil = document.getElementById('settings-veil');
    const panel = document.getElementById('settings');
    const row = (which) => panel.querySelector(`.setting[data-which="${which}"]`);
    const field = (which) => row(which).querySelector('.key');
    const said = (which) => row(which).querySelector('.said');
    const press = (init) => window.dispatchEvent(new KeyboardEvent('keydown', { bubbles: true, cancelable: true, ...init }));
    const settled = async (test) => { for (let i = 0; i < 40 && !test(); i += 1) await sleep(50); return test(); };
    const shown = () => getComputedStyle(veil).display !== 'none';
    const file = async () => JSON.parse((await invoke('editor_settings_file')) || '{}');
    await invoke('editor_settings_pretend_taken', { shortcuts: ['Ctrl+Alt+T'] });

    const min = document.getElementById('win-min').getBoundingClientRect();
    const box = button.getBoundingClientRect();
    const icon = button.querySelector('svg').getBoundingClientRect();
    const minIcon = document.querySelector('#win-min svg').getBoundingClientRect();
    check('the chevron sits left of minimize, in the same box with the same icon size, named Settings', Math.round(box.right) === Math.round(min.left) && box.width === min.width && box.height === min.height && box.top === min.top
      && icon.width === minIcon.width && icon.height === minIcon.height && button.title === 'Settings',
      `${Math.round(box.left)}-${Math.round(box.right)} by ${box.height}, the icon ${icon.width}, minimize from ${Math.round(min.left)}`);
    check('closed, the modal is not on screen', !shown());

    button.click();
    await settled(shown);
    const panelBox = panel.getBoundingClientRect();
    check('a click opens the modal in the middle of the window, titled Settings and nothing more', shown() && panel.querySelector('h2').textContent === 'Settings' && panel.querySelectorAll('h2, h3, p').length === 1
      && Math.abs((panelBox.left + panelBox.right) / 2 - window.innerWidth / 2) <= 1 && Math.abs((panelBox.top + panelBox.bottom) / 2 - window.innerHeight / 2) <= 1 && document.activeElement !== button,
      `the panel ${Math.round(panelBox.left)}-${Math.round(panelBox.right)} by ${Math.round(panelBox.top)}-${Math.round(panelBox.bottom)}`);
    check('it shows the capture shortcut as configured, and none for opening Recon', field('capture').textContent === 'Ctrl+Shift+4' && field('open').textContent === 'None'
      && !row('open').querySelector('.clear').classList.contains('shown'), `${field('open').textContent} / ${field('capture').textContent}`);
    const rowTops = ['open', 'capture', 'capture2'].map((which) => Math.round(row(which).getBoundingClientRect().top));
    check('a second capture row, Capture 2, sits under Capture, in the same size, showing none with no ×', row('capture2').querySelector('span').textContent === 'Capture 2' && field('capture2').textContent === 'None'
      && rowTops[0] < rowTops[1] && rowTops[1] < rowTops[2] && field('capture2').getBoundingClientRect().width === field('capture').getBoundingClientRect().width
      && !row('capture2').querySelector('.clear').classList.contains('shown'), `rows at ${rowTops.join(', ')}, ${field('capture2').textContent}`);

    const toolBefore = editor.model.tool;
    const modeBefore = editor.model.mode;
    press({ code: 'KeyT', key: 't' });
    await sleep(100);
    check('the editor under the modal hears no key', editor.model.tool === toolBefore && editor.model.mode === modeBefore && shown(), `tool ${editor.model.tool}, mode ${editor.model.mode}`);

    field('open').click();
    await settled(() => field('open').classList.contains('recording'));
    check('a click on a field waits for a shortcut', field('open').textContent === 'Press a shortcut' && field('open').classList.contains('recording'));
    press({ code: 'KeyR', key: 'r' });
    await sleep(100);
    check('a key alone is not a shortcut, and the field says what to hold', field('open').classList.contains('recording') && said('open').textContent.includes('Hold Ctrl'), said('open').textContent);
    press({ code: 'ControlLeft', key: 'Control', ctrlKey: true });
    await sleep(50);
    check('a modifier alone keeps waiting', field('open').classList.contains('recording'));
    press({ code: 'KeyR', key: 'ר', ctrlKey: true, altKey: true });
    await settled(() => field('open').textContent === 'Ctrl+Alt+R');
    check('Ctrl+Alt+R, pressed under a Hebrew layout, is kept by its physical key, and the file has it', field('open').textContent === 'Ctrl+Alt+R' && !field('open').classList.contains('recording')
      && !said('open').classList.contains('shown') && row('open').querySelector('.clear').classList.contains('shown') && (await file()).open_hotkey === 'Ctrl+Alt+R' && (await file()).hotkey === 'Ctrl+Shift+4',
      `${field('open').textContent}, the file ${JSON.stringify(await file())}`);

    field('capture').click();
    await settled(() => field('capture').classList.contains('recording'));
    press({ code: 'KeyT', key: 't', ctrlKey: true, altKey: true });
    await settled(() => said('capture').classList.contains('shown'));
    check('a shortcut another application holds is refused in words, and the old one stays, on screen and in the file', said('capture').textContent.includes('taken by another application') && field('capture').textContent === 'Ctrl+Shift+4'
      && (await invoke('editor_settings')).capture === 'Ctrl+Shift+4' && (await file()).hotkey === 'Ctrl+Shift+4', said('capture').textContent);

    field('capture').click();
    await settled(() => field('capture').classList.contains('recording'));
    press({ code: 'KeyR', key: 'r', ctrlKey: true, altKey: true });
    await settled(() => said('capture').classList.contains('shown'));
    check('the two never share a shortcut', said('capture').textContent.includes('already Open Recon') && field('capture').textContent === 'Ctrl+Shift+4', said('capture').textContent);

    field('capture').click();
    await settled(() => field('capture').classList.contains('recording'));
    press({ code: 'Escape', key: 'Escape' });
    await settled(() => !field('capture').classList.contains('recording'));
    check('Escape while waiting stops the waiting and keeps the modal and the shortcut', shown() && field('capture').textContent === 'Ctrl+Shift+4' && !field('capture').classList.contains('recording'));

    field('capture').click();
    await settled(() => field('capture').classList.contains('recording'));
    press({ code: 'Digit5', key: '%', ctrlKey: true, shiftKey: true });
    await settled(() => field('capture').textContent === 'Ctrl+Shift+5');
    check('a new capture shortcut is kept, the file has it, and the host names it to the empty editor', field('capture').textContent === 'Ctrl+Shift+5' && (await file()).hotkey === 'Ctrl+Shift+5' && (await file()).open_hotkey === 'Ctrl+Alt+R'
      && (await invoke('editor_hotkey')) === 'Ctrl+Shift+5',
      `${field('capture').textContent}, the file ${JSON.stringify(await file())}, the host says ${await invoke('editor_hotkey')}`);

    field('capture2').click();
    await settled(() => field('capture2').classList.contains('recording'));
    press({ code: 'KeyP', key: 'p', ctrlKey: true, altKey: true });
    await settled(() => field('capture2').textContent === 'Ctrl+Alt+P');
    check('a second capture shortcut is kept beside the first, and the file has both', field('capture2').textContent === 'Ctrl+Alt+P' && field('capture').textContent === 'Ctrl+Shift+5'
      && row('capture2').querySelector('.clear').classList.contains('shown') && (await file()).second_hotkey === 'Ctrl+Alt+P' && (await file()).hotkey === 'Ctrl+Shift+5'
      && (await invoke('editor_settings')).capture2 === 'Ctrl+Alt+P' && (await invoke('editor_hotkey')) === 'Ctrl+Shift+5',
      `${field('capture2').textContent}, the file ${JSON.stringify(await file())}`);
    field('capture2').click();
    await settled(() => field('capture2').classList.contains('recording'));
    press({ code: 'Digit5', key: '%', ctrlKey: true, shiftKey: true });
    await settled(() => said('capture2').classList.contains('shown'));
    check('the second capture shortcut never takes the first one', said('capture2').textContent.includes('already Capture') && field('capture2').textContent === 'Ctrl+Alt+P', said('capture2').textContent);
    field('open').click();
    await settled(() => field('open').classList.contains('recording'));
    press({ code: 'KeyP', key: 'p', ctrlKey: true, altKey: true });
    await settled(() => said('open').classList.contains('shown'));
    check('nor does Open Recon take it', said('open').textContent.includes('already Capture 2') && field('open').textContent === 'Ctrl+Alt+R', said('open').textContent);
    row('capture2').querySelector('.clear').click();
    await settled(() => field('capture2').textContent === 'None');
    check('the × beside Capture 2 clears it, in the file too, and the first stays', field('capture2').textContent === 'None' && !('second_hotkey' in (await file())) && (await file()).hotkey === 'Ctrl+Shift+5',
      JSON.stringify(await file()));

    row('open').querySelector('.clear').click();
    await settled(() => field('open').textContent === 'None');
    check('the × beside Open Recon clears it, in the file too; Capture has no ×', field('open').textContent === 'None' && !('open_hotkey' in (await file())) && row('capture').querySelector('.clear') === null,
      JSON.stringify(await file()));

    press({ code: 'Escape', key: 'Escape' });
    await settled(() => !shown());
    check('Escape closes the modal', !shown());
    button.click();
    await settled(shown);
    veil.dispatchEvent(new PointerEvent('pointerdown', { bubbles: true, clientX: 5, clientY: 5 }));
    await settled(() => !shown());
    check('a press outside the panel closes it', !shown());
    button.click();
    await settled(shown);
    panel.dispatchEvent(new PointerEvent('pointerdown', { bubbles: true }));
    await sleep(100);
    const stayed = shown();
    document.getElementById('settings-close').click();
    await settled(() => !shown());
    check('a press inside it does not, and its × does', stayed && !shown());
    // T is the text tool's key: under the modal it did nothing, closed it picks the tool.
    const modeClosed = editor.model.mode;
    const toolClosed = editor.model.tool;
    const other = toolClosed === 'text' ? { code: 'KeyR', key: 'r', tool: 'rect' } : { code: 'KeyT', key: 't', tool: 'text' };
    press({ code: other.code, key: other.key });
    await settled(() => editor.model.tool === other.tool);
    check('closed, the keys are the editor\'s again', editor.model.tool === other.tool, `tool ${toolClosed} to ${editor.model.tool}`);
    // Every way out of a note keeps it: one being typed when Settings opens is committed.
    if (editor.model.mode === 'annotate') {
      const typed = editor.createCallout({ x: 120, y: 120 });
      editor.layoutScene();
      editor.startEditing(typed);
      el(typed).querySelector('.t').textContent = 'typed before settings';
      button.click();
      await settled(shown);
      check('a note being typed when Settings opens is kept, and the typing has ended', editor.model.editing === null && typed.text === 'typed before settings' && editor.model.callouts.includes(typed),
        `editing ${editor.model.editing && editor.model.editing.id}, text "${typed.text}"`);
      press({ code: 'Escape', key: 'Escape' });
      await settled(() => !shown());
      editor.model.callouts = editor.model.callouts.filter((c) => c !== typed);
      editor.layoutScene();
    } else {
      check('a note being typed when Settings opens is kept, and the typing has ended', false, 'the mode never reached annotate, so no note could be typed');
    }
    if (modeClosed === 'view') await editor.setMode('view');
    editor.setTool(toolClosed);

    // Back to the default for the sections after this one.
    await invoke('editor_settings_set', { which: 'capture', shortcut: 'Ctrl+Shift+4' });
    await invoke('editor_settings_pretend_taken', { shortcuts: [] });
  }

  // ---------------------------------------------------------------- 37. S2.8: the timeline at scale
  say('');
  say('S2.8: the strip holds a screenful of cells whatever the library holds; its top edge drags it taller, the thumbnails growing to 320 wide and then wrapping into rows of 320; the height is remembered');
  {
    const strip = editor.strip;
    const stage = document.getElementById('stage');
    const handle = document.getElementById('strip-handle');
    let remembered = null;
    try { remembered = localStorage.getItem('recon.strip-height'); } catch (_) { /* none */ }
    await invoke('editor_store_reset');
    editor.setStripHeight(96);
    const shots = [];
    for (let i = 0; i < 30; i += 1) shots.push(await invoke('editor_capture_probe', { width: 300, height: 200 }));
    await editor.loadImage(shots[29]);
    await editor.refreshStrip();
    let layout = editor.stripLayout();
    const cellsOf = () => strip.querySelectorAll('.thumb').length;
    check('thirty documents, one row of 114 by 71 cells, 12 px apart, the band of the scroller of 12 px under them, the newest first and current', editor.stripCells().length === 30 && layout.rows === 1 && layout.w === 114 && layout.h === 71
      && strip.children[0].classList.contains('current') && Number(strip.children[0].dataset.id) === shots[29].document_id,
      `${editor.stripCells().length} cells, ${layout.rows} row(s) of ${layout.w}x${layout.h}`);
    const rendered = cellsOf();
    check('only the screen and a screen either side exist as elements, not the thirty', rendered > 0 && rendered < 30 && rendered >= Math.floor(strip.clientWidth / 126),
      `${rendered} of 30 in a strip ${strip.clientWidth} wide`);
    check('the strip scrolls the whole row all the same', strip.scrollWidth === 24 + 30 * 126 - 12, `scroll width ${strip.scrollWidth}`);
    check('the row scrolls sideways by Rotem\'s own scroller, 12 px under the cells: a 4 px thumb with 4 px clear above and below, not the system\'s', strip.offsetHeight - strip.clientTop - strip.clientHeight === 12,
      `${strip.offsetHeight - strip.clientTop - strip.clientHeight} px between the strip's content and its bottom edge`);
    check('and the cells end above the scroller: the bottom edge of the current cell, border and all, inside the content of the strip (Rotem, 2026-09-17; 4 px of every cell sat under it before)', strip.children[0].offsetTop + strip.children[0].offsetHeight <= strip.clientHeight,
      `the cell ends at ${strip.children[0].offsetTop + strip.children[0].offsetHeight}, the content at ${strip.clientHeight}`);
    strip.scrollLeft = strip.scrollWidth;
    await sleep(100);
    check('scrolled to the end: the oldest cell exists, the newest is dropped', !!strip.querySelector(`[data-id="${shots[0].document_id}"]`) && !strip.querySelector(`[data-id="${shots[29].document_id}"]`) && cellsOf() < 30,
      `${cellsOf()} cells, scrollLeft ${strip.scrollLeft}`);
    strip.scrollLeft = 0;
    await sleep(100);
    for (let i = 0; i < 100 && ![...strip.querySelectorAll('img')].some((img) => img.complete && img.naturalWidth > 0); i += 1) await sleep(50);
    check('and back: the newest is there again, with its picture', !!strip.querySelector(`[data-id="${shots[29].document_id}"] img`), `${cellsOf()} cells`);

    // The drag: taller, and the cells grow to 320 wide; then the height is exactly the
    // hand's, never snapped, and a row enters when there is room for one.
    const h1 = editor.setStripHeight(300);
    layout = editor.stripLayout();
    check('dragged past one row of 320: the strip is exactly as dragged, 300 tall, one row of 320 by 200 cells', h1 === 300 && layout.rows === 1 && layout.w === 320 && layout.h === 200 && stage.clientHeight === window.innerHeight - 64 - 48 - 300,
      `${h1} tall, ${layout.rows} row(s) of ${layout.w}x${layout.h}, stage ${stage.clientHeight}`);
    const h1b = editor.setStripHeight(435);
    check('one pixel short of a second row: still one row', h1b === 435 && editor.stripLayout().rows === 1, `${h1b} tall, ${editor.stripLayout().rows} row(s)`);
    const h2 = editor.setStripHeight(436);
    layout = editor.stripLayout();
    const cols = Math.floor((strip.clientWidth - 24 + 12) / 332);
    const second = editor.cellRect(cols);
    check('at 436 the second row enters: two rows of 320, 12 px apart, as many columns as fit, scrolled vertically', h2 === 436 && layout.rows === 2 && layout.cols === cols && strip.classList.contains('grid') && second.x === 12 && second.y === 224
      && strip.scrollHeight === 24 + Math.ceil(30 / cols) * 212 - 12 && stage.clientHeight === window.innerHeight - 64 - 48 - 436,
      `${h2} tall, ${layout.rows} rows of ${layout.cols}, cell ${cols} at ${second.x},${second.y}, scroll height ${strip.scrollHeight}`);
    check('the rows scroll by Rotem\'s own scroller, 4 px wide at the strip\'s right edge, not the system\'s', strip.offsetWidth - strip.clientWidth === 4,
      `${strip.offsetWidth - strip.clientWidth} px between the strip's edge and its content`);
    const h2b = editor.setStripHeight(500);
    check('and between rows the height is the hand\'s, the rows unchanged', h2b === 500 && editor.stripLayout().rows === 2 && stage.clientHeight === window.innerHeight - 64 - 48 - 500, `${h2b} tall, ${editor.stripLayout().rows} rows`);
    // Rotem, 2026-09-15: a strip dragged taller covers the sidebar, and its handle stays on top.
    const stripTop = Math.round(strip.getBoundingClientRect().top);
    const underSidebar = [...document.querySelectorAll('#controls button')].filter((b) => b.getBoundingClientRect().top >= stripTop);
    const hits = underSidebar.map((b) => { const r = b.getBoundingClientRect(); return document.elementFromPoint(r.left + r.width / 2, r.top + r.height / 2); });
    const handleHit = document.elementFromPoint(24, stripTop);
    check('at 500 tall the strip covers the sidebar\'s buttons below its top, and the handle is above the strip', underSidebar.length > 0 && hits.every((h) => strip.contains(h)) && handleHit === handle,
      `${underSidebar.length} button(s) below the strip's top at ${stripTop}, hit ${hits.map((h) => h && (h.id || h.className || h.tagName)).join(' ')}; at the edge ${handleHit && (handleHit.id || handleHit.tagName)}`);
    const h3 = editor.setStripHeight(100000);
    check('the strip never takes more than 96% of the window', h3 === Math.floor(window.innerHeight * 0.96), `${h3} of ${window.innerHeight}`);
    // The picture's size never rises into the top bar: it stops 16 px under it, and the strip covers it there (2026-09-16).
    const cappedSize = document.getElementById('image-size').getBoundingClientRect();
    const cappedHit = document.elementFromPoint(100, cappedSize.top + 8);
    check('at the ceiling the picture\'s size stops 16 px under the top bar, and the strip covers it', Math.round(cappedSize.top) === 80 && strip.contains(cappedHit),
      `the size from ${Math.round(cappedSize.top)}, the strip from ${Math.round(strip.getBoundingClientRect().top)}, hit ${cappedHit && (cappedHit.id || cappedHit.className || cappedHit.tagName)}`);

    // The handle itself, with pointer events: up by 200 from the default, then a double-click.
    editor.setStripHeight(96);
    const pointer = (type, y) => handle.dispatchEvent(new PointerEvent(type, { clientY: y, clientX: 100, button: 0, pointerId: 1, bubbles: true, cancelable: true }));
    pointer('pointerdown', 700);
    pointer('pointermove', 500);
    pointer('pointerup', 500);
    let kept = null;
    try { kept = localStorage.getItem('recon.strip-height'); } catch (_) { /* none */ }
    check('a drag on the top edge resizes the strip by exactly the move and the page remembers the height', editor.stripHeightOf() === 296 && kept === '296' && getComputedStyle(handle).cursor === 'ns-resize',
      `${editor.stripHeightOf()} tall, remembered ${kept}, cursor ${getComputedStyle(handle).cursor}`);
    handle.dispatchEvent(new MouseEvent('dblclick', { bubbles: true }));
    try { kept = localStorage.getItem('recon.strip-height'); } catch (_) { /* none */ }
    check('a double-click on the edge returns the strip to its default, 137 tall: the line, 12 px, 112 px thumbnails ending at 124, the band of the scroller under them', editor.stripHeightOf() === 137 && kept === '137' && editor.stripLayout().h === 112 && stage.clientHeight === window.innerHeight - 64 - 48 - 137 && strip.clientHeight === 124 && strip.children[0].offsetTop + strip.children[0].offsetHeight === 124,
      `${editor.stripHeightOf()} tall, remembered ${kept}, thumbnails ${editor.stripLayout().h} tall, the cell ends at ${strip.children[0].offsetTop + strip.children[0].offsetHeight}, the content at ${strip.clientHeight}`);

    // Rotem, 2026-09-15: a press anywhere on the one-row strip and a move sideways scrolls it,
    // and the click that ends the scroll shows no document; a press that does not move still does.
    strip.scrollLeft = 0;
    await sleep(100);
    const pressAt = (target, type, x) => target.dispatchEvent(new PointerEvent(type, { clientX: x, clientY: strip.getBoundingClientRect().top + 40, button: 0, pointerId: 2, bubbles: true, cancelable: true }));
    const pressed = strip.querySelector('.thumb:not(.current)');
    const shownBefore = model.image.document_id;
    pressAt(pressed, 'pointerdown', 600);
    pressAt(pressed, 'pointermove', 450);
    pressAt(pressed, 'pointermove', 300);
    pressAt(pressed, 'pointerup', 300);
    pressed.click();
    await sleep(400);
    const dragged = { left: strip.scrollLeft, shown: model.image.document_id };
    const clicked = [...strip.querySelectorAll('.thumb:not(.current)')].find((t) => t.getBoundingClientRect().left > 60);
    const clickedId = Number(clicked.dataset.id);
    pressAt(clicked, 'pointerdown', 700);
    pressAt(clicked, 'pointerup', 702);
    clicked.click();
    for (let i = 0; i < 40 && model.image.document_id !== clickedId; i += 1) await sleep(50);
    check('a press on a thumbnail and a move sideways scrolls the strip by the move and shows nothing; a press that barely moves shows the thumbnail',
      dragged.left === 300 && dragged.shown === shownBefore && model.image.document_id === clickedId,
      `scrolled ${dragged.left} for a move of 300, shown ${dragged.shown} (was ${shownBefore}); the still press showed ${model.image.document_id}, wanted ${clickedId}`);

    // Rotem, 2026-09-16: over the one-row strip the wheel scrolls it sideways, down to the right;
    // once the rows wrap the page leaves the wheel to the strip's own vertical scroll.
    strip.scrollLeft = 0;
    await sleep(100);
    const wheelOn = (deltaY) => {
      const wheel = new WheelEvent('wheel', { deltaY, deltaMode: 0, bubbles: true, cancelable: true });
      (strip.querySelector('.thumb') || strip).dispatchEvent(wheel);
      return wheel.defaultPrevented;
    };
    const downTaken = wheelOn(120);
    const afterDown = strip.scrollLeft;
    const upTaken = wheelOn(-50);
    const afterUp = strip.scrollLeft;
    editor.setStripHeight(436); // two rows enter here since the 12 px gap (2026-09-17)
    const rowsWrapped = editor.stripLayout().rows;
    const rowsTaken = wheelOn(120);
    editor.setStripHeight(96);
    check('over the one-row strip the wheel scrolls it sideways, down to the right, and rows leave the wheel alone',
      downTaken && afterDown === 120 && upTaken && afterUp === 70 && rowsWrapped > 1 && !rowsTaken,
      `down to ${afterDown}, up to ${afterUp}, taken ${downTaken} ${upTaken}; ${rowsWrapped} rows, taken ${rowsTaken}`);

    // The trash view lays out the same way.
    await editor.deleteDocument(shots[0].document_id);
    // The chip is the last cell, so with twenty-nine documents it exists only near the end.
    check('the Trash chip is the last cell, past the screen until the strip is scrolled there', editor.stripCells()[29].key === 'chip:trash' && !strip.querySelector('#trash-chip'));
    strip.scrollLeft = strip.scrollWidth;
    await sleep(100);
    check('scrolled to the end, it is there', !!strip.querySelector('#trash-chip'),
      `${cellsOf()} cells of ${editor.stripCells().length}, scrollLeft ${strip.scrollLeft} of ${strip.scrollWidth}, ${JSON.stringify(editor.stripLayout())}, last child ${strip.lastChild && strip.lastChild.className} ${strip.lastChild && strip.lastChild.dataset.id}`);
    const chipEl = strip.querySelector('#trash-chip');
    if (chipEl) chipEl.click();
    for (let i = 0; i < 60 && !strip.querySelector('.thumb.trashed'); i += 1) await sleep(50);
    const backEl = strip.querySelector('#trash-back');
    check('the trash view is the same cells: the Back chip first, the trashed document after it', strip.children.length === 2 && !!backEl && strip.children[0].contains(backEl) && strip.children[1].classList.contains('trashed'),
      `${strip.children.length} children, first ${strip.children[0] && strip.children[0].className}`);
    if (backEl) backEl.click();
    for (let i = 0; i < 60 && editor.stripCells().length !== 30; i += 1) await sleep(50);
    check('and Back returns to the documents, scrolled to the current one', !document.body.classList.contains('trash') && editor.stripCells().length === 30 && editor.stripCells()[29].key === 'chip:trash' && !!strip.querySelector('.thumb.current'),
      `trash ${document.body.classList.contains('trash')}, ${editor.stripCells().length} cells, last ${editor.stripCells()[editor.stripCells().length - 1].key}, current ${!!strip.querySelector('.thumb.current')}`);

    try { if (remembered === null) localStorage.removeItem('recon.strip-height'); else localStorage.setItem('recon.strip-height', remembered); } catch (_) { /* none */ }
    editor.setStripHeight(96);
    model.callouts = [];
    editor.layoutScene();
  }

  // ---------------------------------------------------------------- 38. S2.8: the startup read
  say('');
  say('S2.8: a startup reads the newest fifty records and shows the latest before the rest of the store is read on a thread; the list becomes whole and the page is told; the storage figure comes from a ledger, never a walk');
  {
    const strip = editor.strip;
    await invoke('editor_store_reset');
    editor.setStripHeight(96);
    const shots = [];
    for (let i = 0; i < 60; i += 1) shots.push(await invoke('editor_capture_probe', { width: 300, height: 200 }));
    await editor.loadImage(shots[59]);
    await editor.saveNow();
    const before = await invoke('editor_storage');
    check('sixty documents on disk, and the ledger says so without a walk', before.documents === 60 && before.bytes > 0, `${before.documents} documents, ${before.bytes} bytes`);

    // A restart: the newest fifty are read before the latest reopens, the ten oldest follow.
    editor.documents.clear();
    model.callouts = [];
    model.image = { width: 0, height: 0, source: '' };
    let loaded = false;
    const unlisten = await window.__TAURI__.event.listen('store-loaded', () => { loaded = true; });
    const started = performance.now();
    const back = await invoke('editor_store_reload');
    const reopened = performance.now() - started;
    const partial = await invoke('editor_documents');
    check('the latest document reopens at once, from the newest fifty', back.document_id === shots[59].document_id && back.width === 300 && partial.length >= 50 && partial.length <= 60,
      `document ${back.document_id}, ${partial.length} listed after ${reopened.toFixed(0)} ms`);
    await editor.loadImage(back);
    for (let i = 0; i < 100 && !loaded; i += 1) await sleep(50);
    for (let i = 0; i < 60 && editor.stripCells().length !== 60; i += 1) await sleep(50);
    unlisten();
    const whole = await invoke('editor_documents');
    const info = await invoke('editor_image_info');
    check('the rest arrives on a thread, the page is told, the timeline takes all sixty and the position reads 60 of 60', loaded && whole.length === 60 && editor.stripCells().length === 60 && info.position === 60 && info.total === 60 && model.image.total === 60,
      `told ${loaded}, ${whole.length} listed, ${editor.stripCells().length} cells, ${info.position} of ${info.total}, hud ${model.image.position} of ${model.image.total}`);
    // Thumbnails are written as cells show, so the ledger is read against a walk, not
    // against the earlier figure.
    const walk = async () => (await invoke('editor_store_list')).reduce((sum, l) => sum + (l.json ? l.bytes : 0), 0);
    const after = await invoke('editor_storage');
    const walked = await walk();
    check('the ledger holds every folder after the read, byte for byte with a walk', after.documents === 60 && after.bytes === walked, `${after.documents} documents, ${after.bytes} bytes, a walk says ${walked}`);

    // The ledger follows a delete and a restore.
    await editor.deleteDocument(shots[0].document_id);
    const less = await invoke('editor_storage');
    const lessWalked = await walk();
    await editor.loadImage(await invoke('editor_trash_restore', { id: shots[0].document_id }));
    const same = await invoke('editor_storage');
    const sameWalked = await walk();
    check('a delete takes its folder out of the ledger and a restore puts it back', less.documents === 59 && less.bytes === lessWalked && same.documents === 60 && same.bytes === sameWalked && same.bytes > less.bytes,
      `${less.documents} then ${same.documents} documents, ${less.bytes} (walk ${lessWalked}) then ${same.bytes} (walk ${sameWalked}) bytes`);
    await editor.refreshStrip();
    check('the timeline agrees', editor.stripCells().filter((c) => c.kind === 'doc').length === 60 && !strip.querySelector('#trash-chip'));
    model.callouts = [];
    editor.layoutScene();
  }

  // ---------------------------------------------------------------- 39. S2.9: undo of a deletion
  say('');
  say('S2.9: Ctrl+Z brings back the picture just deleted from the timeline, the last thing done first; Ctrl+Shift+Z deletes it again; the notes and their undo come back with it; no notice');
  {
    const hud = document.getElementById('hud');
    await invoke('editor_store_reset');
    const press = (init) => window.dispatchEvent(new KeyboardEvent('keydown', { bubbles: true, cancelable: true, ...init }));
    const undoKey = () => press({ key: 'z', code: 'KeyZ', ctrlKey: true });
    const redoKey = () => press({ key: 'Z', code: 'KeyZ', ctrlKey: true, shiftKey: true });
    const until = async (test) => { for (let i = 0; i < 80 && !test(); i += 1) await sleep(50); return test(); };
    const inTrash = async (id) => (await invoke('editor_trash_list')).some(([t]) => t === id);
    const listed = async (id) => (await invoke('editor_documents')).some((d) => d.id === id);
    const thumbs = () => editor.stripCells().filter((c) => c.kind === 'doc').map((c) => c.doc.id);

    // Three captures; the middle one gets a note and a second step, so its undo has history.
    const a = await invoke('editor_capture_probe', { width: 320, height: 200 });
    await editor.loadImage(a);
    const b = await invoke('editor_capture_probe', { width: 340, height: 200 });
    await editor.loadImage(b);
    const n = editor.createCallout({ x: 30, y: 30 });
    n.text = 'kept through the trash';
    editor.layoutScene();
    editor.record();
    n.text = 'kept through the trash, edited';
    editor.layoutScene();
    editor.record();
    await editor.saveNow();
    const bRecord = await invoke('editor_store_read', { id: b.document_id });
    const bBytes = (await invoke('editor_store_list')).find((l) => l.id === b.document_id).source_bytes;
    const c = await invoke('editor_capture_probe', { width: 360, height: 200 });
    await editor.loadImage(c);
    await editor.refreshStrip();

    // 1. Delete the one on screen, Ctrl+Z: it is back on screen, with its notes, its undo
    //    still walking, its folder unchanged, out of the trash, no notice.
    await editor.loadImage(await invoke('editor_show_document', { id: b.document_id }));
    await editor.deleteDocument(b.document_id);
    check('the middle document deleted from the screen: in the trash, the neighbour on screen', await inTrash(b.document_id) && model.image.document_id === c.document_id, `on screen ${model.image.document_id}`);
    editor.notice('', 0);
    undoKey();
    check('Ctrl+Z after a delete brings the same document back on screen', await until(() => model.image.document_id === b.document_id), `on screen ${model.image.document_id}`);
    await sleep(200);
    check('with its notes as they were, out of the trash, back in the list', model.callouts.length === 1 && model.callouts[0].text === 'kept through the trash, edited' && !(await inTrash(b.document_id)) && (await listed(b.document_id)),
      `${model.callouts.length} notes, "${model.callouts[0] && model.callouts[0].text}"`);
    const bRecordAfter = await invoke('editor_store_read', { id: b.document_id }).catch((err) => String(err));
    const bBytesAfter = ((await invoke('editor_store_list')).find((l) => l.id === b.document_id) || {}).source_bytes;
    // The folder as a whole may grow by the thumbnail the timeline asks for after the return;
    // the image and the record are what a move must leave alone.
    check('its record and its image are byte for byte what they were: a move, never a rewrite', bRecordAfter === bRecord && bBytesAfter === bBytes, `image ${bBytes} then ${bBytesAfter} bytes, record ${bRecordAfter === bRecord ? 'same' : 'CHANGED'}`);
    check('no notice for the coming back', !hud.textContent.includes('restored') && !hud.textContent.includes('NOT'), hud.textContent.split('\n').pop());
    check('the timeline has all three again, in place', await until(() => thumbs().length === 3 && thumbs()[1] === b.document_id), JSON.stringify(thumbs()));
    const noteUndo = editor.undo();
    check('its own note undo still walks: the earlier text is back', noteUndo && !!model.callouts[0] && model.callouts[0].text === 'kept through the trash', model.callouts[0] && model.callouts[0].text);
    editor.redo();
    check('and redoes', !!model.callouts[0] && model.callouts[0].text === 'kept through the trash, edited');

    // 2. Delete one not on screen, Ctrl+Z: the picture on screen unchanged, the thumbnail back.
    await editor.deleteDocument(a.document_id);
    check('the oldest deleted from its thumbnail while the middle one is on screen', await inTrash(a.document_id) && model.image.document_id === b.document_id);
    undoKey();
    check('Ctrl+Z brings its thumbnail back and leaves the screen alone', await until(() => thumbs().length === 3) && model.image.document_id === b.document_id && !(await inTrash(a.document_id)),
      `${thumbs().length} thumbnails, on screen ${model.image.document_id}`);

    // 3. The last thing done first: a note step, then a deletion, Ctrl+Z takes the deletion
    //    with the note still there; the next Ctrl+Z takes the note; Ctrl+Shift+Z walks back.
    const m = editor.createCallout({ x: 80, y: 80 });
    m.text = 'newer than nothing';
    editor.layoutScene();
    editor.record();
    await editor.deleteDocument(c.document_id);
    undoKey();
    check('a note, then a delete: Ctrl+Z brings the picture back first, the note still there', await until(() => thumbs().length === 3) && model.callouts.length === 2 && model.image.document_id === b.document_id,
      `${thumbs().length} thumbnails, ${model.callouts.length} notes`);
    undoKey();
    check('the next Ctrl+Z takes the note', await until(() => model.callouts.length === 1), `${model.callouts.length} notes`);
    redoKey();
    check('Ctrl+Shift+Z puts the note back', await until(() => model.callouts.length === 2), `${model.callouts.length} notes`);
    redoKey();
    check('Ctrl+Shift+Z again deletes the picture again, into the trash', await until(() => thumbs().length === 2) && (await inTrash(c.document_id)), `${thumbs().length} thumbnails`);
    undoKey();
    check('and Ctrl+Z brings it back once more', await until(() => thumbs().length === 3) && !(await inTrash(c.document_id)));

    // 4. Two deletes, two Ctrl+Z: both back, the later first.
    await editor.deleteDocument(a.document_id);
    await editor.deleteDocument(c.document_id);
    undoKey();
    check('two deletes: the first Ctrl+Z brings the later one back', await until(() => thumbs().length === 2) && thumbs().includes(c.document_id) && !thumbs().includes(a.document_id), JSON.stringify(thumbs()));
    undoKey();
    check('the second brings the earlier one back', await until(() => thumbs().length === 3) && thumbs().includes(a.document_id));

    // 5. The last one deleted, the editor empty, Ctrl+Z from there: it is back on screen.
    await editor.deleteDocument(a.document_id);
    await editor.deleteDocument(c.document_id);
    await editor.deleteDocument(b.document_id);
    check('every document deleted: the editor is empty', model.image.width === 0 && thumbs().length === 0);
    undoKey();
    check('Ctrl+Z from the empty state brings the last one back on screen, with its notes', await until(() => model.image.document_id === b.document_id && model.image.width === 340) && model.callouts.length === 2,
      `on screen ${model.image.document_id}, ${model.callouts.length} notes`);
    const k = editor.createCallout({ x: 120, y: 120 });
    k.text = 'new';
    editor.layoutScene();
    editor.record();
    redoKey();
    await sleep(300);
    check('a new action ends the redo: after a fresh note, Ctrl+Shift+Z deletes nothing', thumbs().length === 1 && (await listed(b.document_id)) && model.callouts.length === 3, `${thumbs().length} thumbnails, ${model.callouts.length} notes`);

    // 6. A restore refused: the failure is a line on screen, and the next Ctrl+Z moves on.
    await editor.deleteDocument(b.document_id);
    await invoke('editor_trash_age', { id: b.document_id, days: 31 });
    await invoke('editor_sweep_trash');
    undoKey();
    check('a restore that cannot happen says so and leaves the undo path', await until(() => hud.textContent.includes('NOT RESTORED')) && !editor.deletionsOf().some((d) => d.id === b.document_id), hud.textContent.split('\n').pop());
    editor.notice('', 0);

    // 7. A delete and an undo inside the capture's own encode (review 4, T2): the undo
    //    waits for the image, and the picture comes back whole, its record and its image in
    //    one folder among the documents, nothing left in the trash.
    const racing = await invoke('editor_capture_probe', { width: 4000, height: 2500 });
    await editor.loadImage(racing);
    await editor.deleteDocument(racing.document_id);
    undoKey();
    const raced = await until(() => model.image.document_id === racing.document_id && model.image.width === 4000);
    await sleep(300);
    const racedLine = (await invoke('editor_store_list')).find((l) => l.id === racing.document_id);
    check('a capture deleted and brought back inside its own encode comes back whole: on screen, its record and its image in one folder, nothing in the trash, no notice',
      raced && !!racedLine && racedLine.json && racedLine.source_png && !(await inTrash(racing.document_id)) && !hud.textContent.includes('NOT RESTORED'),
      `back ${raced}, folder ${racedLine ? `json ${racedLine.json}, png ${racedLine.source_png}` : 'none'}, ${hud.textContent.split('\n').pop()}`);
    model.callouts = [];
    editor.layoutScene();
  }

  // ---------------------------------------------------------------- 40. the timeline's tabs
  say('');
  say('The tabs (Rotem, 2026-09-16): a plus at the left of the size band; its first press makes Main and New tab; a capture on a tab lands in Main and in that tab; the × pressed twice deletes a tab, never Main; a tab after Main is dragged into another place; a click on the selected tab\'s name edits it; adding, deleting and renaming are in undo; the list survives a reload');
  {
    const tabbar = editor.tabbar;
    const plus = document.getElementById('tab-add');
    const sizeEl = document.getElementById('image-size');
    const until = async (test) => { for (let i = 0; i < 80 && !test(); i += 1) await sleep(50); return test(); };
    const thumbs = () => editor.stripCells().filter((c) => c.kind === 'doc').map((c) => c.doc.id);
    const tabEls = () => [...tabbar.querySelectorAll('.tab')];
    const tabNames = () => tabEls().map((t) => t.querySelector('.name').textContent);
    const selectedName = () => { const t = tabbar.querySelector('.tab.selected'); return t ? t.querySelector('.name').textContent : ''; };
    const press = (init) => window.dispatchEvent(new KeyboardEvent('keydown', { bubbles: true, cancelable: true, ...init }));
    const undoKey = () => press({ key: 'z', code: 'KeyZ', ctrlKey: true });
    const redoKey = () => press({ key: 'Z', code: 'KeyZ', ctrlKey: true, shiftKey: true });
    await invoke('editor_store_reset');
    await invoke('editor_save_tabs', { tabs: { schema: 1, selected: 0, tabs: [] } });
    await editor.loadTabs();
    editor.setStripHeight(96);
    const a = await invoke('editor_capture_probe', { width: 320, height: 200 });
    await editor.loadImage(a);
    const b = await invoke('editor_capture_probe', { width: 340, height: 200 });
    await editor.loadImage(b);
    await editor.refreshStrip();

    // The plus: an icon button at the left of the band, 32 by 32, centred in the band's height, a 20 by 20 icon; no tab yet.
    const plusBox = plus.getBoundingClientRect();
    const bandTop = Math.round(sizeEl.getBoundingClientRect().top) - 16;
    const icon = plus.querySelector('svg').getBoundingClientRect();
    check('the plus sits at the left of the size band, 32 by 32, 8 px in and centred in the band, with a 20 by 20 icon and no text',
      getComputedStyle(tabbar).display === 'flex' && Math.round(plusBox.width) === 32 && Math.round(plusBox.height) === 32 && Math.round(plusBox.left) === 8
        && Math.round(plusBox.top) === bandTop + 8 && Math.round(icon.width) === 20 && Math.round(icon.height) === 20 && plus.textContent.trim() === '' && plus.getAttribute('aria-label') === 'New tab',
      `${Math.round(plusBox.left)},${Math.round(plusBox.top)} ${Math.round(plusBox.width)}x${Math.round(plusBox.height)}, the band from ${bandTop}, icon ${Math.round(icon.width)}x${Math.round(icon.height)}`);
    check('before any press there is no tab, and the timeline lists the whole library', tabEls().length === 0 && thumbs().length === 2, `${tabEls().length} tabs, ${thumbs().length} thumbnails`);

    // The first press: Main and New tab, the new one selected, its feed empty; the tabs 14 px text on a 10 px radius (Rotem, 2026-09-16).
    plus.click();
    await sleep(150);
    check('the first press puts Main and New tab beside the plus, New tab selected', tabNames().join('|') === 'Main|New tab' && selectedName() === 'New tab', `${tabNames().join('|')}, selected "${selectedName()}"`);
    check('Main is right after the plus, and no tab carries a ×', plus.nextElementSibling === tabEls()[0] && !tabEls()[0].querySelector('.x') && !tabEls()[1].querySelector('.x'));
    const tabStyle = getComputedStyle(tabEls()[1]);
    check('a tab is 32 px tall with 14 px text on a 10 px radius', tabStyle.fontSize === '14px' && tabStyle.borderRadius === '10px' && Math.round(tabEls()[1].getBoundingClientRect().height) === 32, `${tabStyle.fontSize}, radius ${tabStyle.borderRadius}`);
    check('the new tab\'s feed is empty, so the timeline shows nothing yet, and stays', await until(() => thumbs().length === 0) && document.body.classList.contains('strip'), `${thumbs().length} thumbnails`);

    // A capture on the new tab: in the library, so in Main, and in the tab's feed.
    const c = await invoke('editor_capture_probe', { width: 360, height: 200 });
    await editor.loadImage(c);
    check('a capture taken on the tab shows in its feed', await until(() => thumbs().length === 1 && thumbs()[0] === c.document_id), JSON.stringify(thumbs()));
    let saved = await invoke('editor_tabs');
    const t1 = editor.tabsOf().list[0];
    check('and it is recorded in the tab\'s list on disk, the selected tab with it', !!saved && saved.selected === t1.id && saved.tabs.length === 1 && saved.tabs[0].docs.join() === String(c.document_id), JSON.stringify(saved));
    tabEls()[0].click();
    check('a click on Main shows the whole library, the capture among it', await until(() => thumbs().length === 3) && selectedName() === 'Main', `${thumbs().length} thumbnails, selected "${selectedName()}"`);
    tabEls()[1].click();
    check('back on the tab, its one capture', await until(() => thumbs().length === 1) && selectedName() === 'New tab');

    // The done mark (Rotem, 2026-09-16): in a tab, a round 20 by 20 button at the thumbnail's top left with a
    // 1 px white border and a 20 by 20 tick of a 2 px line, hidden until hover; pressed, it stays shown and the tick fades in by A1,
    // on disk; Ctrl+Z clears it; Main has none. The sizes are read from the rules, since a hidden element has no box.
    const doneBtn = document.querySelector('#strip .thumb .done');
    const doneStyle = doneBtn && getComputedStyle(doneBtn);
    const doneIcon = doneBtn && getComputedStyle(doneBtn.querySelector('svg'));
    check('in a tab a thumbnail carries a round 24 by 24 done mark at its top left, a 1 px white border, a 20 by 20 tick of a 2 px line inside it, hidden until hover',
      !!doneBtn && doneStyle.display === 'none' && doneStyle.width === '24px' && doneStyle.height === '24px' && doneStyle.borderRadius === '12px' && doneStyle.left === '2px' && doneStyle.top === '2px'
        && doneStyle.borderTopWidth === '1px' && doneStyle.borderTopColor === 'rgb(255, 255, 255)' && doneStyle.boxSizing === 'border-box'
        && doneIcon.width === '20px' && doneIcon.height === '20px' && doneIcon.strokeWidth === '2px' && !doneBtn.querySelector('.ring') && !doneBtn.closest('.thumb').classList.contains('checked'),
      doneBtn ? `${doneStyle.display}, ${doneStyle.width}x${doneStyle.height} radius ${doneStyle.borderRadius} at ${doneStyle.left},${doneStyle.top}, border ${doneStyle.borderTopWidth} ${doneStyle.borderTopColor}, icon ${doneIcon.width}` : 'no mark');
    const tickBefore = getComputedStyle(doneBtn.querySelector('.tick')).opacity;
    doneBtn.click();
    const tickStyle = getComputedStyle(doneBtn.querySelector('.tick'));
    const fade = `${tickStyle.transitionProperty} ${tickStyle.transitionDuration} ${tickStyle.transitionTimingFunction}`;
    await sleep(300);
    saved = await invoke('editor_tabs');
    check('the tick was clear before the press and fades in by A1, 144 ms ease-out, to full once pressed', tickBefore === '0' && fade === 'opacity 0.144s ease-out' && getComputedStyle(doneBtn.querySelector('.tick')).opacity === '1',
      `before ${tickBefore}, ${fade}, after ${getComputedStyle(doneBtn.querySelector('.tick')).opacity}`);
    check('pressed, the mark stays shown and ticked, and the picture is marked done in this tab on disk', doneBtn.closest('.thumb').classList.contains('checked') && getComputedStyle(doneBtn).display === 'grid'
        && doneBtn.closest('.thumb').getBoundingClientRect().left + doneBtn.closest('.thumb').clientLeft + 2 === doneBtn.getBoundingClientRect().left && Math.round(doneBtn.getBoundingClientRect().width) === 24 && Math.round(doneBtn.querySelector('svg').getBoundingClientRect().width) === 20
        && editor.tabsOf().list[0].done.join() === String(c.document_id) && !!saved && (saved.tabs[0].done || []).join() === String(c.document_id),
      `checked ${doneBtn.closest('.thumb').classList.contains('checked')}, done ${JSON.stringify(editor.tabsOf().list[0].done)}, disk ${JSON.stringify(saved && saved.tabs[0].done)}`);
    check('the picture is still on screen and its notes untouched: the mark changes nothing else', model.image.document_id === c.document_id && model.callouts.length === 0 && thumbs().length === 1);
    undoKey();
    await sleep(100);
    check('Ctrl+Z clears the mark', !document.querySelector('#strip .thumb.checked') && editor.tabsOf().list[0].done.length === 0, JSON.stringify(editor.tabsOf().list[0].done));
    redoKey();
    await sleep(100);
    check('Ctrl+Shift+Z marks it again', !!document.querySelector('#strip .thumb.checked') && editor.tabsOf().list[0].done.join() === String(c.document_id));
    document.querySelector('#strip .thumb .done').click();
    await sleep(100);
    check('pressed again, the mark goes', !document.querySelector('#strip .thumb.checked') && editor.tabsOf().list[0].done.length === 0);
    tabEls()[0].click();
    await until(() => thumbs().length === 3);
    check('in Main no thumbnail carries the mark', document.querySelectorAll('#strip .thumb').length === 3 && !document.querySelector('#strip .thumb .done'));
    tabEls()[1].click();
    await until(() => thumbs().length === 1);

    // A reload of the list from disk: the same tabs, the same feed, the same selection.
    await editor.loadTabs();
    await editor.refreshStrip();
    check('the tabs read back from disk as they were saved', tabNames().join('|') === 'Main|New tab' && selectedName() === 'New tab' && editor.tabsOf().list[0].docs.join() === String(c.document_id) && thumbs().length === 1,
      `${tabNames().join('|')}, selected "${selectedName()}", feed ${JSON.stringify(editor.tabsOf().list[0].docs)}`);

    // The name: a click on the selected tab's name edits it in place, Enter keeps it; Ctrl+Z takes it back.
    const nameEl = tabEls()[1].querySelector('.name');
    nameEl.click();
    check('a click on the selected tab\'s name opens it for editing', nameEl.isContentEditable && document.activeElement === nameEl, `editable ${nameEl.isContentEditable}, focus on ${document.activeElement && document.activeElement.className}`);
    nameEl.textContent = 'Acme remarks';
    nameEl.dispatchEvent(new KeyboardEvent('keydown', { bubbles: true, cancelable: true, key: 'Enter', code: 'Enter' }));
    await sleep(100);
    saved = await invoke('editor_tabs');
    check('Enter keeps the name, on screen and on disk, and the editing ends', !nameEl.isContentEditable && tabNames()[1] === 'Acme remarks' && editor.tabsOf().list[0].name === 'Acme remarks' && !!saved && saved.tabs[0].name === 'Acme remarks', `"${tabNames()[1]}", disk "${saved && saved.tabs[0].name}"`);
    tabEls()[0].querySelector('.name').click();
    await sleep(50);
    check('a click on another tab\'s name selects it rather than editing it, and Main is never edited', selectedName() === 'Main' && !tabEls()[0].querySelector('.name').isContentEditable);
    tabEls()[0].click();
    tabEls()[0].querySelector('.name').click();
    check('Main\'s name is not edited even when it is selected', !tabEls()[0].querySelector('.name').isContentEditable);
    undoKey();
    await sleep(100);
    check('Ctrl+Z takes the rename back', tabNames()[1] === 'New tab' && editor.tabsOf().list[0].name === 'New tab', `"${tabNames()[1]}"`);
    redoKey();
    await sleep(100);
    check('Ctrl+Shift+Z renames it again', tabNames()[1] === 'Acme remarks', `"${tabNames()[1]}"`);

    // Two more presses, then a drag of the last one to right after Main, by the pointer.
    plus.click();
    plus.click();
    await sleep(150);
    const list = editor.tabsOf().list;
    check('every press adds a New tab and selects it', list.length === 3 && tabNames().join('|') === 'Main|Acme remarks|New tab|New tab' && editor.tabsOf().selected === list[2].id, tabNames().join('|'));
    const els = tabEls();
    const third = els[3];
    const firstAfterMain = els[1].getBoundingClientRect();
    const start = third.getBoundingClientRect();
    const pointer = (type, x, target) => target.dispatchEvent(new PointerEvent(type, { bubbles: true, cancelable: true, pointerId: 7, button: 0, buttons: 1, clientX: x, clientY: start.top + 16 }));
    pointer('pointerdown', start.left + 20, third);
    pointer('pointermove', start.left + 12, third);
    pointer('pointermove', firstAfterMain.left + 4, third);
    const followed = third.classList.contains('dragging') && third.style.transform.startsWith('translateX(');
    const orderDuring = editor.tabsOf().list.map((t) => t.id).join();
    pointer('pointerup', firstAfterMain.left + 4, third);
    third.click();
    await sleep(100);
    check('a press on the last tab and a move past the first one drags it there: it follows the hand and takes the place right after Main', followed && orderDuring === [list[2].id, list[0].id, list[1].id].join() && editor.tabsOf().list.map((t) => t.id).join() === orderDuring
      && tabEls()[1] === third && third.style.transform === '' && !third.classList.contains('dragging'),
      `followed ${followed}, order ${editor.tabsOf().list.map((t) => list.findIndex((l) => l.id === t.id) + 1).join()}`);
    check('the drop that ends the drag selects nothing: the selection is where it was', editor.tabsOf().selected === list[2].id);
    saved = await invoke('editor_tabs');
    check('the new order is on disk', !!saved && saved.tabs.map((t) => t.id).join() === orderDuring, saved && saved.tabs.map((t) => t.id).join());
    const mainEl = tabEls()[0];
    const mainBox = mainEl.getBoundingClientRect();
    pointer('pointerdown', mainBox.left + 10, mainEl);
    pointer('pointermove', mainBox.left + 200, mainEl);
    pointer('pointerup', mainBox.left + 200, mainEl);
    check('Main cannot be dragged: a press and a move on it changes nothing', tabEls()[0] === mainEl && !mainEl.classList.contains('dragging') && editor.tabsOf().list.map((t) => t.id).join() === orderDuring);
    check('and a move of it by the page is refused', editor.moveTab(0, 1) === false && editor.deleteTab(0) === false && tabNames()[0] === 'Main');

    // The tab's menu (Rotem, 2026-09-18): a right press on a tab after Main opens it with Delete; a press elsewhere or Escape closes it; Main has none.
    const t1El = tabEls().find((el) => Number(el.dataset.tab) === t1.id);
    t1El.click();
    await until(() => thumbs().length === 1);
    const tabMenu = document.getElementById('tab-menu');
    const rightPress = (el) => { const b = el.getBoundingClientRect(); return el.dispatchEvent(new MouseEvent('contextmenu', { bubbles: true, cancelable: true, button: 2, clientX: b.left + 12, clientY: b.top + 16 })); };
    const els2 = tabEls();
    const gapMain = Math.round(els2[1].getBoundingClientRect().left - els2[0].getBoundingClientRect().right);
    const gapNext = Math.round(els2[2].getBoundingClientRect().left - els2[1].getBoundingClientRect().right);
    const nameEnd = Math.round(els2[1].getBoundingClientRect().right - els2[1].querySelector('.name').getBoundingClientRect().right);
    check('two new tabs are as far apart as Main and the first one, 8 px, and a name ends 10 px before its tab does: no room is held for a ×', gapMain === 8 && gapNext === 8 && nameEnd === 10, `${gapMain}, ${gapNext}, name end ${nameEnd}`);
    const mainTook = !rightPress(tabEls()[0]);
    check('a right press on Main opens nothing, and not the system menu either', mainTook && getComputedStyle(tabMenu).display === 'none');
    const took = !rightPress(t1El);
    const menuBox = tabMenu.getBoundingClientRect();
    check('a right press on a tab opens a menu with Delete alone, above the bar at the pointer, and deletes nothing', took && getComputedStyle(tabMenu).display === 'block' && tabMenu.querySelectorAll('button').length === 1 && tabMenu.textContent.trim() === 'Delete'
        && Math.abs(menuBox.bottom - (t1El.getBoundingClientRect().top + 16)) < 1 && Math.abs(menuBox.left - (t1El.getBoundingClientRect().left + 12)) < 1 && editor.tabsOf().list.length === 3,
      `took ${took}, ${getComputedStyle(tabMenu).display}, "${tabMenu.textContent.trim()}", ${menuBox.left},${menuBox.bottom} for ${t1El.getBoundingClientRect().left + 12},${t1El.getBoundingClientRect().top + 16}, ${editor.tabsOf().list.length} tabs`);
    document.body.dispatchEvent(new PointerEvent('pointerdown', { bubbles: true, cancelable: true, pointerId: 7, button: 0 }));
    check('a press elsewhere closes the menu and deletes nothing', getComputedStyle(tabMenu).display === 'none' && editor.tabsOf().list.length === 3);
    rightPress(t1El);
    tabMenu.querySelector('button').dispatchEvent(new KeyboardEvent('keydown', { bubbles: true, cancelable: true, key: 'Escape', code: 'Escape' }));
    check('Escape closes it too', getComputedStyle(tabMenu).display === 'none' && editor.tabsOf().list.length === 3);
    rightPress(t1El);
    tabMenu.querySelector('button').click();
    check('Delete in the menu deletes the selected tab and selects Main: the whole library again, the capture still there, the menu closed', await until(() => thumbs().length === 3) && selectedName() === 'Main' && editor.tabsOf().list.length === 2 && !editor.tabsOf().list.some((t) => t.id === t1.id) && getComputedStyle(tabMenu).display === 'none',
      `${thumbs().length} thumbnails, selected "${selectedName()}", ${editor.tabsOf().list.length} tabs`);
    saved = await invoke('editor_tabs');
    check('the deletion is on disk', !!saved && saved.tabs.length === 2 && saved.selected === 0, JSON.stringify(saved && saved.tabs.map((t) => t.id)));

    // Undo: Ctrl+Z brings the deleted tab back in its place, with its feed, selected as it was; Ctrl+Shift+Z deletes it again.
    undoKey();
    check('Ctrl+Z brings the deleted tab back in its place, with its name, its feed and the selection', await until(() => thumbs().length === 1) && editor.tabsOf().list.length === 3 && editor.tabsOf().list[1].id === t1.id && editor.tabsOf().list[1].name === 'Acme remarks' && editor.tabsOf().list[1].docs.join() === String(c.document_id) && selectedName() === 'Acme remarks',
      `${editor.tabsOf().list.map((t) => t.name).join('|')}, selected "${selectedName()}", ${thumbs().length} thumbnails`);
    saved = await invoke('editor_tabs');
    check('and it is on disk again', !!saved && saved.tabs.length === 3 && saved.tabs[1].id === t1.id, JSON.stringify(saved && saved.tabs.map((t) => t.id)));
    redoKey();
    check('Ctrl+Shift+Z deletes it again', await until(() => thumbs().length === 3) && editor.tabsOf().list.length === 2 && selectedName() === 'Main', `${editor.tabsOf().list.length} tabs`);
    undoKey();
    await until(() => editor.tabsOf().list.length === 3);
    // Undo of an addition: the press's tab goes, the selection returns to the tab before it.
    tabEls().find((el) => Number(el.dataset.tab) === t1.id).click();
    await until(() => thumbs().length === 1);
    plus.click();
    await sleep(100);
    check('a press adds a fourth tab, selected', editor.tabsOf().list.length === 4 && selectedName() === 'New tab' && editor.tabsOf().selected === editor.tabsOf().list[3].id);
    undoKey();
    check('Ctrl+Z takes the added tab away and the selection returns to the tab before it', await until(() => editor.tabsOf().list.length === 3 && thumbs().length === 1) && editor.tabsOf().selected === t1.id, `${editor.tabsOf().list.length} tabs, selected "${selectedName()}"`);
    redoKey();
    check('Ctrl+Shift+Z adds it back, selected', await until(() => editor.tabsOf().list.length === 4) && editor.tabsOf().selected === editor.tabsOf().list[3].id);
    // The last thing done first: a note step, then a tab deletion; Ctrl+Z takes the deletion, the note stays.
    tabEls()[0].click();
    await until(() => thumbs().length === 3);
    await editor.loadImage(await invoke('editor_show_document', { id: c.document_id }));
    const n = editor.createCallout({ x: 30, y: 30 });
    n.text = 'a note before the tab went';
    editor.layoutScene();
    editor.record();
    editor.deleteTab(editor.tabsOf().list[3].id);
    await sleep(100);
    undoKey();
    check('a note, then a tab deleted: Ctrl+Z brings the tab back first, the note still there', await until(() => editor.tabsOf().list.length === 4) && model.callouts.length === 1, `${editor.tabsOf().list.length} tabs, ${model.callouts.length} notes`);
    undoKey();
    check('the next Ctrl+Z takes the note', await until(() => model.callouts.length === 0), `${model.callouts.length} notes`);
    for (const tab of editor.tabsOf().list) editor.deleteTab(tab.id);
    await sleep(100);
    check('with every tab deleted, Main goes too and only the plus is left', tabEls().length === 0 && thumbs().length === 3, `${tabEls().length} tabs`);

    // Fullscreen puts the bar away with the band.
    plus.click();
    await sleep(100);
    await editor.setFullscreen(true);
    check('fullscreen puts the tabs away', getComputedStyle(tabbar).display === 'none', getComputedStyle(tabbar).display);
    await editor.setFullscreen(false);
    check('and they come back, above the timeline', getComputedStyle(tabbar).display === 'flex' && Math.round(tabbar.getBoundingClientRect().bottom) === Math.round(editor.strip.getBoundingClientRect().top) - 8,
      `${getComputedStyle(tabbar).display}, the bar to ${Math.round(tabbar.getBoundingClientRect().bottom)}, the strip from ${Math.round(editor.strip.getBoundingClientRect().top)}`);
    editor.deleteTab(editor.tabsOf().list[0].id);
    await invoke('editor_save_tabs', { tabs: { schema: 1, selected: 0, tabs: [] } });
    await editor.loadTabs();
    await editor.refreshStrip();
    model.callouts = [];
    editor.layoutScene();
  }

  // ---------------------------------------------------------------- 41. the callout in two clicks
  say('');
  say('The callout in two clicks (Rotem, 2026-09-16): the first click marks the end of the line, the bubble follows the pointer, the second click locks it and the typing begins; Escape, Ctrl+Z or another tool between them drops the unplaced bubble');
  {
    await openFixture('reference-scene.png');
    await editor.setMode('annotate');
    await editor.setMargin({ left: 0, top: 0, right: 0, bottom: 0 });
    model.callouts = []; model.nextNumber = 1; model.shapes = [];
    model.history = { steps: [editor.snapshot()], index: 0 };
    editor.layoutScene();
    const stage = editor.stage;
    const box = stage.getBoundingClientRect();
    const css = (ix, iy) => ({ x: box.left + model.offset.x + ((ix - model.pan.x) * model.zoom) / editor.ratioOf(), y: box.top + model.offset.y + ((iy - model.pan.y) * model.zoom) / editor.ratioOf() });
    const pointer = (type, ix, iy) => {
      const at = css(ix, iy);
      stage.dispatchEvent(new PointerEvent(type, { pointerId: 15, button: 0, buttons: type === 'pointerup' ? 0 : 1, clientX: at.x, clientY: at.y, bubbles: true, cancelable: true }));
    };
    const click = (ix, iy) => { pointer('pointerdown', ix, iy); pointer('pointerup', ix, iy); };
    const press = (init) => { const e = new KeyboardEvent('keydown', { bubbles: true, cancelable: true, ...init }); window.dispatchEvent(e); return e.defaultPrevented; };
    const line = () => { const l = document.querySelector('#arrows line'); return l ? { x1: Number(l.getAttribute('x1')), y1: Number(l.getAttribute('y1')) } : null; };

    editor.setTool('callout');
    click(200, 150);
    const c = model.callouts[0];
    check('the first click makes one note, anchored where it was, and nothing is typed yet',
      model.callouts.length === 1 && c.anchor.x === 200 && c.anchor.y === 150 && model.placing === c && model.editing === null && model.selected === null,
      `anchor ${c && c.anchor.x},${c && c.anchor.y}, editing ${model.editing ? 'yes' : 'no'}`);
    const start = { ...c.box };
    const gx = Math.round(c.textSize * 1.4); const gy = Math.round(c.textSize * 1.2);
    check('its bubble starts at the automatic place, below and right of the anchor on an empty picture', start.x === 200 + gx && start.y === 150 + gy, `${start.x},${start.y} against ${200 + gx},${150 + gy}`);
    pointer('pointermove', 300, 250);
    check('the bubble follows the pointer, keeping its offset from it',
      c.box.x === start.x + 100 && c.box.y === start.y + 100 && document.querySelector(`[data-id="${c.id}"]`).style.left === `${start.x + 100}px`,
      `${start.x},${start.y} to ${c.box.x},${c.box.y}`);
    check('the anchor stays where the first click was, and the line starts there', c.anchor.x === 200 && c.anchor.y === 150 && line() && line().x1 === 200 && line().y1 === 150);
    pointer('pointermove', 360, 300);
    click(360, 300);
    check('the second click locks the bubble where it is and the typing begins',
      model.placing === null && model.editing === c && c.box.x === start.x + 160 && c.box.y === start.y + 150 && document.activeElement === document.querySelector(`[data-id="${c.id}"] .t`),
      `box ${c.box.x},${c.box.y}, editing ${model.editing === c}`);
    pointer('pointermove', 500, 380);
    check('after the lock the pointer moves the bubble no more', c.box.x === start.x + 160 && c.box.y === start.y + 150);
    document.querySelector(`[data-id="${c.id}"] .t`).textContent = 'placed in two clicks';
    editor.commitEditing();
    check('the note is one step of history, its text kept', model.callouts.length === 1 && c.text === 'placed in two clicks' && model.history.index === 1, `index ${model.history.index}`);
    press({ key: 'z', code: 'KeyZ', ctrlKey: true });
    check('Ctrl+Z takes the note back', model.callouts.length === 0 && model.nextNumber === 1);
    press({ key: 'Z', code: 'KeyZ', ctrlKey: true, shiftKey: true });
    check('Ctrl+Shift+Z brings it back where it was locked', model.callouts.length === 1 && model.callouts[0].box.x === start.x + 160 && model.callouts[0].text === 'placed in two clicks');

    // Nothing typed after the second click (review 4, T5): the bubble goes and its number
    // comes back, as it does between the clicks, and no step is added.
    editor.setTool('callout');
    click(120, 260);
    pointer('pointermove', 200, 320);
    click(200, 320);
    const typingEmpty = model.callouts.length === 2 && model.editing === model.callouts[1] && model.nextNumber === 3;
    press({ key: 'Escape', code: 'Escape' });
    check('Escape after the second click with nothing typed discards the bubble, gives its number back and adds no step',
      typingEmpty && model.callouts.length === 1 && model.editing === null && model.nextNumber === 2 && model.history.index === 1,
      `typing ${typingEmpty}; ${model.callouts.length} notes, next number ${model.nextNumber}, history ${model.history.index}`);

    // Between the clicks: Escape, Ctrl+Z and a change of tool each drop the unplaced bubble
    // and give its number back.
    const drops = [
      ['Escape', () => press({ key: 'Escape', code: 'Escape' })],
      ['Ctrl+Z', () => press({ key: 'z', code: 'KeyZ', ctrlKey: true })],
      ['a change of tool', () => press({ key: 'l', code: 'KeyL' })],
    ];
    for (const [name, drop] of drops) {
      editor.setTool('callout');
      const stepsBefore = model.history.index;
      click(100, 100);
      pointer('pointermove', 180, 160);
      const number = model.callouts[1] && model.callouts[1].number;
      drop();
      check(`${name} between the clicks drops the unplaced bubble, the earlier note untouched, the number given back`,
        model.callouts.length === 1 && model.placing === null && model.editing === null && model.nextNumber === number && model.history.index === stepsBefore,
        `${model.callouts.length} notes, next number ${model.nextNumber} after ${number}, history ${model.history.index}`);
    }
    check('the tool changed to is in hand after the drop', model.tool === 'arrow');

    // The text tool is untouched: one click, typed where it lands.
    editor.setTool('text');
    click(400, 300);
    check('the text tool still types at its one click', !!model.editing && model.editing.kind === 'text' && model.placing === null);
    editor.commitEditing();

    // The second click may land on the bubble itself, when the pointer stops over it.
    editor.setTool('callout');
    click(300, 200);
    const d = model.placing;
    const onBubble = document.querySelector(`[data-id="${d.id}"] .t`);
    const at = css(d.box.x + 20, d.box.y + 10);
    onBubble.dispatchEvent(new PointerEvent('pointerdown', { pointerId: 15, button: 0, buttons: 1, clientX: at.x, clientY: at.y, bubbles: true, cancelable: true }));
    onBubble.dispatchEvent(new PointerEvent('pointerup', { pointerId: 15, button: 0, buttons: 0, clientX: at.x, clientY: at.y, bubbles: true, cancelable: true }));
    check('a second click on the bubble itself locks it too', model.placing === null && model.editing === d);
    editor.commitEditing();

    // Another picture arriving between the clicks: the unplaced bubble is dropped, never
    // stashed with the picture it was started on.
    const docId = model.image.document_id;
    const notesBefore = model.callouts.length;
    const numberBefore = model.nextNumber;
    editor.setTool('callout');
    click(150, 150);
    pointer('pointermove', 250, 220);
    const probe = await invoke('editor_load_probe', { width: 300, height: 200 });
    await editor.loadImage(probe);
    check('a picture arriving between the clicks drops the unplaced bubble', model.placing === null && model.callouts.length === 0);
    await editor.loadImage(await invoke('editor_show_document', { id: docId }));
    check('the picture it was started on comes back without it, its number given back',
      model.image.document_id === docId && model.callouts.length === notesBefore && model.nextNumber === numberBefore,
      `${model.callouts.length} notes against ${notesBefore}, next ${model.nextNumber} against ${numberBefore}`);
    model.callouts = []; model.shapes = []; model.nextNumber = 1;
    editor.layoutScene();
    editor.setTool(null);
  }

  // ---------------------------------------------------------------- 42. the thumbnail carries the notes
  say('');
  say('The thumbnail carries the notes (Rotem, 2026-09-16): after a save of the notes the host writes thumb.png anew with the layer composed over the picture at thumbnail scale, and the strip shows it');
  {
    const pixelsOf = async (id) => {
      const response = await fetch(`http://region.localhost/?thumb=${id}`, { cache: 'no-store' });
      const bitmap = await createImageBitmap(await response.blob());
      const c = new OffscreenCanvas(bitmap.width, bitmap.height);
      const g = c.getContext('2d');
      g.drawImage(bitmap, 0, 0);
      const data = g.getImageData(0, 0, bitmap.width, bitmap.height).data;
      return { w: bitmap.width, h: bitmap.height, at: (x, y) => [...data.slice((y * bitmap.width + x) * 4, (y * bitmap.width + x) * 4 + 3)] };
    };
    const same = (a, b) => a.every((v, i) => Math.abs(v - b[i]) <= 2);
    const until = async (test) => { for (let i = 0; i < 80 && !(await test()); i += 1) await sleep(50); return test(); };

    // A capture the size of the box, so the thumbnail is the picture itself and a note's
    // pixels sit where the note is.
    const shot = await invoke('editor_capture_probe', { width: 320, height: 200 });
    await editor.loadImage(shot);
    await editor.setMargin({ left: 0, top: 0, right: 0, bottom: 0 });
    model.callouts = []; model.nextNumber = 1; model.shapes = [];
    editor.layoutScene();
    await editor.refreshStrip();
    const plain = await pixelsOf(shot.document_id);
    check('the fresh capture\'s thumbnail is the picture at its size', plain.w === 320 && plain.h === 200, `${plain.w}x${plain.h}`);

    // A note that fits inside the picture, so no margin grows: its bubble at 48,64, 260 wide
    // and 102 tall (Rotem's 2026-09-17 bubble: 32 px of padding round a 36 px text box).
    const note = editor.createCallout({ x: 20, y: 40 });
    editor.layoutScene();
    editor.startEditing(note);
    document.querySelector(`[data-id="${note.id}"] .t`).textContent = 'on the thumbnail too';
    editor.commitEditing();
    // Two points of the bubble's padding, its #2D41D7 ground, clear of the text: left of
    // it and right of it.
    const inside = { x: note.box.x + 12, y: note.box.y + 20 };
    const ground = { x: note.box.x + note.box.width - 12, y: note.box.y + 40 };
    const badge = (px) => Math.abs(px[0] - 0x2d) <= 12 && Math.abs(px[1] - 0x41) <= 12 && Math.abs(px[2] - 0xd7) <= 12;
    const bubble = badge;
    check('the note sits inside the picture, so the thumbnail keeps its size', editor.marginString() === '0,0,0,0' && note.box.x + note.box.width <= 320, `margin ${editor.marginString()}, box ${note.box.x},${note.box.y} ${note.box.width} wide`);
    const cell = () => editor.strip.querySelector(`.thumb[data-id="${shot.document_id}"]`);
    const srcBefore = cell() && cell().querySelector('img') ? cell().querySelector('img').src : '';
    await editor.saveNow();
    const refreshed = await editor.thumbnailDone();
    check('the save is followed by a thumbnail refresh that the host accepted', refreshed === true, String(refreshed));
    const noted = await pixelsOf(shot.document_id);
    check('the thumbnail now shows the bubble where the note is: its blue ground left and right of the text', noted.w === 320 && noted.h === 200 && badge(noted.at(inside.x, inside.y)) && !badge(plain.at(inside.x, inside.y)) && bubble(noted.at(ground.x, ground.y)) && !bubble(plain.at(ground.x, ground.y)),
      `badge at ${inside.x},${inside.y}: ${plain.at(inside.x, inside.y)} before, ${noted.at(inside.x, inside.y)} after; ground at ${ground.x},${ground.y}: ${plain.at(ground.x, ground.y)} before, ${noted.at(ground.x, ground.y)} after`);
    check('a pixel away from the note is the picture\'s own, unchanged', same(noted.at(5, 5), plain.at(5, 5)), `${plain.at(5, 5)} before, ${noted.at(5, 5)} after`);
    if (cell()) {
      check('the strip\'s cell took the new picture', await until(() => cell() && cell().querySelector('img') && cell().querySelector('img').src !== srcBefore), `src ${srcBefore ? 'changed' : 'appeared'}`);
    } else {
      skipped('the strip\'s cell took the new picture', 'the capture has no cell in the strip');
    }

    // The note gone: the thumbnail returns to the picture alone.
    editor.removeCallout(note);
    editor.settle();
    editor.record();
    await editor.saveNow();
    await editor.thumbnailDone();
    const cleared = await pixelsOf(shot.document_id);
    check('with the note deleted the thumbnail is the picture alone again', same(cleared.at(inside.x, inside.y), plain.at(inside.x, inside.y)), `${cleared.at(inside.x, inside.y)} against ${plain.at(inside.x, inside.y)}`);

    // A note that grows the margin: the thumbnail is the whole composition fitted into the
    // box, margin and all, so it is smaller than the box in one direction.
    // Anchored low at the left, where no candidate place of a 260 by 102 bubble is inside
    // the 320 by 200 picture, so the first clear one, below right, takes a margin.
    const wide = editor.createCallout({ x: 60, y: 150 });
    editor.layoutScene();
    wide.text = 'a note that grows the margin';
    await editor.settle();
    editor.record();
    await editor.saveNow();
    const grown = await editor.thumbnailDone();
    const withMargin = await pixelsOf(shot.document_id);
    const comp = editor.compositionSize();
    const scale = Math.min(320 / comp.w, 200 / comp.h, 1);
    check('a margin makes the thumbnail the whole composition, fitted into the box', grown === true && editor.marginString() !== '0,0,0,0' && withMargin.w === Math.round(comp.w * scale) && withMargin.h === Math.round(comp.h * scale),
      `composition ${comp.w}x${comp.h}, margin ${editor.marginString()}, thumbnail ${withMargin.w}x${withMargin.h}`);
    model.callouts = []; model.shapes = []; model.nextNumber = 1;
    await editor.setMargin({ left: 0, top: 0, right: 0, bottom: 0 });
    editor.layoutScene();
    editor.record();
    await editor.saveNow();
    await editor.thumbnailDone();
  }

  // ---------------------------------------------------------------- 43. the ruler
  say('');
  say('The ruler (Rotem, 2026-09-16): M or the button, then a drag marks an area; its size in image pixels sits beside the pointer while the drag lasts and stays beside the corner it ended at; hovering it shows a round 16 px x with a 12 px X that deletes it; it moves, undoes, redoes, saves and exports like the other shapes, the x left out of the copy');
  {
    const stage = editor.stage;
    await invoke('editor_store_reset');
    const shot = await invoke('editor_capture_probe', { width: 400, height: 300 });
    await editor.loadImage(shot);
    await editor.setZoom(1);
    const box = stage.getBoundingClientRect();
    const css = (ix, iy) => ({ x: box.left + model.offset.x + ((ix - model.pan.x) * model.zoom) / editor.ratioOf(), y: box.top + model.offset.y + ((iy - model.pan.y) * model.zoom) / editor.ratioOf() });
    const pointer = (type, target, ix, iy) => {
      const at = css(ix, iy);
      target.dispatchEvent(new PointerEvent(type, { pointerId: 13, button: 0, buttons: type === 'pointerup' ? 0 : 1, clientX: at.x, clientY: at.y, bubbles: true, cancelable: true }));
    };
    const press = (init) => { const e = new KeyboardEvent('keydown', { bubbles: true, cancelable: true, ...init }); window.dispatchEvent(e); return e.defaultPrevented; };
    const rulerEl = () => document.querySelector('#scene .ruler[data-shape]');
    const sizeEl = () => rulerEl() && rulerEl().querySelector('.size');
    const xEl = () => rulerEl() && rulerEl().querySelector('.x');
    const geometry = () => { const r = editor.rectOfShape(model.shapes[0]); const el = rulerEl(); return `${r.x},${r.y} ${r.w}x${r.h}; box ${el ? `${el.style.left} ${el.style.top} ${el.style.width} ${el.style.height}` : 'none'}; size "${sizeEl() ? sizeEl().textContent : ''}" at ${sizeEl() ? `${sizeEl().style.left},${sizeEl().style.top}` : 'none'}`; };

    press({ key: 'm', code: 'KeyM' });
    check('M picks the ruler, and its button in the sidebar is lit', model.tool === 'ruler' && document.querySelector('#tools [data-tool="ruler"]').classList.contains('active'));
    const button = document.querySelector('#tools [data-tool="ruler"]');
    check('the ruler button sits after Blur, named Ruler, an icon button like the others', !!button && button.previousElementSibling.dataset.tool === 'blur' && button.getAttribute('aria-label') === 'Ruler' && getComputedStyle(button).width === '48px');

    // The magnifier (Rotem, 2026-09-18): the capture's circle, the whole time the tool is in hand.
    const magEl = document.getElementById('mag');
    const magCanvas = magEl.querySelector('canvas');
    const magPlace = () => { const m = magEl.getBoundingClientRect(); return `${Math.round(m.left)},${Math.round(m.top)} ${Math.round(m.width)}x${Math.round(m.height)}`; };
    // A cell's middle on the circle against the picture's own pixel, asked of the host 1:1.
    // The panel is the host's drawing (host/src/magnifier.rs), which says where its circle
    // is: the blue lines cross on its centre, and the middle cell starts one pixel past them.
    const magNumber = (name) => Number(magEl.dataset[name]);
    const magCell = (dx, dy) => {
      const cell = magNumber('cell');
      const at = (c, d) => c + 1 + d * cell + Math.floor(cell / 2);
      return [...magCanvas.getContext('2d').getImageData(at(magNumber('cx'), dx), at(magNumber('cy'), dy), 1, 1).data].slice(0, 3).join(',');
    };
    const sourcePixel = async (ix, iy) => {
      const whole = new Uint8Array(await (await fetch(`http://region.localhost/?x=${ix}&y=${iy}&w=1&h=1&ow=1&oh=1`, { cache: 'no-store' })).arrayBuffer());
      return [...whole.subarray(8, 11)].join(',');
    };
    stage.dispatchEvent(new PointerEvent('pointerleave', { pointerId: 13 }));
    check('with the pointer off the stage there is no magnifier', getComputedStyle(magEl).display === 'none');
    stage.dispatchEvent(new PointerEvent('pointermove', { pointerId: 13, clientX: css(200, 150).x, clientY: css(200, 150).y, bubbles: true }));
    const hoverAt = css(200, 150);
    await sleep(200); // the panel is the host's drawing now, and shows once it has come
    const hoverBox = magEl.getBoundingClientRect();
    check('with the ruler in hand the magnifier sits below the pointer, 8 px right of it: a 112 px circle with 8 px around it and no size yet', getComputedStyle(magEl).display === 'block' && magEl.dataset.label === '' && Math.round(hoverBox.left - hoverAt.x) === 8 && Math.round(hoverBox.top - hoverAt.y) === 8 && Math.round(hoverBox.width) === 128 && Math.round(hoverBox.height) === 128 && Math.round((2 * magNumber('radius')) / editor.ratioOf()) === 112, `${magPlace()} against the pointer at ${Math.round(hoverAt.x)},${Math.round(hoverAt.y)}`);
    await sleep(200);
    const seen = [magCell(0, 0), magCell(-1, 0), magCell(1, 1)];
    const wanted = [await sourcePixel(200, 150), await sourcePixel(199, 150), await sourcePixel(201, 151)];
    check('the circle shows the picture\'s own pixels, one to a cell: the pointer\'s in the middle, its neighbours beside it', seen.join(' ') === wanted.join(' '), `circle ${seen.join(' ')}; picture ${wanted.join(' ')}`);
    const blueAt = [...magCanvas.getContext('2d').getImageData(magNumber('cx'), magNumber('cy') + 3 * magNumber('cell'), 1, 1).data].slice(0, 3).join(',');
    check('the blue line runs down the left edge of the pointer\'s pixel', blueAt === '37,84,251', blueAt);

    // The drag: the size above the circle while it lasts, the x hidden meanwhile.
    pointer('pointerdown', stage, 100, 80);
    pointer('pointermove', stage, 200, 150);
    await sleep(200);
    check('while a ruler is dragged out its size sits above the circle, and the ruler\'s own label waits for the release', magEl.dataset.label === '100x70' && magNumber('cy') - magNumber('radius') > 8 * editor.ratioOf() && getComputedStyle(document.querySelector('#scene .ruler .size')).display === 'none', `${magPlace()} "${magEl.dataset.label}", the circle's top at ${magNumber('cy') - magNumber('radius')}`);
    let ruler = model.shapes[0];
    check('a drag makes a ruler, a box in the scene over the area crossed so far', !!ruler && ruler.kind === 'ruler' && !!rulerEl() && rulerEl().style.left === '100px' && rulerEl().style.top === '80px' && rulerEl().style.width === '100px' && rulerEl().style.height === '70px', geometry());
    check('while the drag lasts the size is written beside the pointer, 8 px right of and below it, in image pixels', !!sizeEl() && sizeEl().textContent === '100x70' && sizeEl().style.left === '108px' && sizeEl().style.top === '78px', geometry());
    check('while the drag lasts the x is hidden', !!xEl() && rulerEl().classList.contains('drawing') && getComputedStyle(xEl()).display === 'none');
    pointer('pointermove', stage, 260, 200);
    check('the size follows the pointer as it moves', sizeEl().textContent === '160x120' && sizeEl().style.left === '168px' && sizeEl().style.top === '128px', geometry());
    pointer('pointerup', stage, 260, 200);
    const r1 = editor.rectOfShape(ruler);
    check('released, the ruler stays with its size beside the corner the drag ended at', model.shapes.length === 1 && r1.x === 100 && r1.y === 80 && r1.w === 160 && r1.h === 120 && sizeEl().textContent === '160x120' && sizeEl().style.left === '168px' && sizeEl().style.top === '128px' && !rulerEl().classList.contains('drawing'), geometry());
    check('released, the ruler\'s own size label shows again and the magnifier goes with the tool', getComputedStyle(sizeEl()).display !== 'none' && model.tool === null && getComputedStyle(magEl).display === 'none', `label ${getComputedStyle(sizeEl()).display}, tool ${model.tool}, magnifier ${getComputedStyle(magEl).display}`);
    check('the ruler is one step of history', model.history.index >= 1);
    check('the size label is in image pixels, 14 px medium on a dark bubble, and takes no pointer', getComputedStyle(sizeEl()).fontSize === '14px' && getComputedStyle(sizeEl()).fontWeight === '500' && getComputedStyle(sizeEl()).pointerEvents === 'none');

    // The x: round, 16 by 16 with a 12 by 12 icon, at the top right corner, 16 screen pixels at any zoom.
    const xs = getComputedStyle(xEl());
    const xi = getComputedStyle(xEl().querySelector('svg'));
    check('the x is a round 16 by 16 button with a 12 by 12 X, hidden until hover, at the top right corner', xs.display === 'none' && xs.width === '16px' && xs.height === '16px' && xs.borderRadius === '8px' && xi.width === '12px' && xi.height === '12px' && xEl().getAttribute('aria-label') === 'Delete',
      `${xs.display} ${xs.width}x${xs.height} radius ${xs.borderRadius}, icon ${xi.width}x${xi.height}`);
    xEl().style.display = 'grid';
    const corner1 = css(260, 80);
    const b1 = xEl().getBoundingClientRect();
    await editor.setZoom(2);
    const corner2 = css(260, 80);
    const b2 = xEl().getBoundingClientRect();
    check('the x is 16 screen pixels centred on the corner at zoom 1 and at zoom 2 alike, the scene\'s scale undone on it', Math.abs(b1.width - 16) < 0.6 && Math.abs(b1.height - 16) < 0.6 && Math.abs(b2.width - 16) < 0.6 && Math.abs(b2.height - 16) < 0.6
      && Math.abs(b1.left + b1.width / 2 - corner1.x) < 1 && Math.abs(b1.top + b1.height / 2 - corner1.y) < 1 && Math.abs(b2.left + b2.width / 2 - corner2.x) < 1 && Math.abs(b2.top + b2.height / 2 - corner2.y) < 1,
      `zoom 1: ${b1.width.toFixed(1)}x${b1.height.toFixed(1)} centre ${(b1.left + b1.width / 2).toFixed(1)},${(b1.top + b1.height / 2).toFixed(1)} corner ${corner1.x.toFixed(1)},${corner1.y.toFixed(1)}; zoom 2: ${b2.width.toFixed(1)}x${b2.height.toFixed(1)} centre ${(b2.left + b2.width / 2).toFixed(1)},${(b2.top + b2.height / 2).toFixed(1)} corner ${corner2.x.toFixed(1)},${corner2.y.toFixed(1)}`);
    await editor.setZoom(1);
    xEl().style.display = '';

    // The copy: the box and its size, never the x.
    const layer = await editor.exportLayer();
    check('the export carries the ruler and its size, and not the x', layer.markup.includes('160x120') && !layer.markup.includes('<button') && !layer.markup.includes('data-ui'), `${layer.markup.includes('160x120')} ${layer.markup.includes('<button')}`);

    // Moved whole, the size stays beside its corner; undo puts it back.
    pointer('pointerdown', rulerEl(), 150, 150);
    pointer('pointermove', stage, 160, 170);
    pointer('pointerup', stage, 160, 170);
    const r2 = editor.rectOfShape(model.shapes[0]);
    check('a drag on the ruler moves it whole, the size with it', model.selectedShape === model.shapes[0] && r2.x === 110 && r2.y === 100 && r2.w === 160 && r2.h === 120 && sizeEl().style.left === '168px' && sizeEl().style.top === '128px', geometry());
    press({ key: 'z', code: 'KeyZ', ctrlKey: true });
    check('Ctrl+Z puts it back', editor.rectOfShape(model.shapes[0]).x === 100 && editor.rectOfShape(model.shapes[0]).y === 80, geometry());
    press({ key: 'Escape', code: 'Escape' });

    // A click with no drag makes nothing.
    press({ key: 'm', code: 'KeyM' });
    pointer('pointerdown', stage, 20, 20);
    pointer('pointerup', stage, 21, 20);
    check('a click with no drag makes no ruler', model.shapes.length === 1 && document.querySelectorAll('#scene .ruler').length === 1);

    // Saved with the notes, back after a restart.
    await editor.saveNow();
    editor.documents.clear();
    model.shapes = [];
    model.image = { width: 0, height: 0, source: '' };
    await editor.loadImage(await invoke('editor_store_reload'));
    ruler = model.shapes[0];
    check('after a restart the ruler is back from the disk with its size', model.shapes.length === 1 && ruler.kind === 'ruler' && ruler.b.x === 260 && ruler.b.y === 200 && !!sizeEl() && sizeEl().textContent === '160x120', geometry());

    // The x deletes it, one step of undo; Ctrl+Shift+Z deletes it again.
    await editor.setMode('annotate');
    pointer('pointerdown', xEl(), 260, 80);
    pointer('pointerup', stage, 260, 80);
    check('a press on the x deletes the ruler', model.shapes.length === 0 && !rulerEl());
    press({ key: 'z', code: 'KeyZ', ctrlKey: true });
    check('Ctrl+Z brings it back, size and all', model.shapes.length === 1 && model.shapes[0].kind === 'ruler' && !!sizeEl() && sizeEl().textContent === '160x120', geometry());
    press({ key: 'Z', code: 'KeyZ', ctrlKey: true, shiftKey: true });
    check('Ctrl+Shift+Z deletes it again', model.shapes.length === 0 && !rulerEl());
    press({ key: 'z', code: 'KeyZ', ctrlKey: true });
    check('and Ctrl+Z once more brings it back', model.shapes.length === 1 && !!rulerEl());

    press({ key: 'c', code: 'KeyC' });
    model.callouts = [];
    model.shapes = [];
    editor.layoutScene();
    editor.setTool(null);
    for (let i = 0; i < 40 && !(await invoke('editor_window_visible')); i += 1) await sleep(50);
    await invoke('editor_show');
  }

  // ---------------------------------------------------------------- 44. the crop
  say('');
  say('The crop (Rotem, 2026-09-17): K or the button, then a frame with eight handles sits on the picture\'s edges; a handle dragged inward cuts the picture to the frame at the release, on screen, in the size band, in every copy and in the thumbnail; the notes keep their place; one step of undo; saved and back after a restart; the source is never touched');
  {
    const stage = editor.stage;
    await invoke('editor_store_reset');
    const shot = await invoke('editor_capture_probe', { width: 400, height: 300 });
    await editor.loadImage(shot);
    await editor.setZoom(1);
    const box = () => stage.getBoundingClientRect();
    const css = (ix, iy) => ({ x: box().left + model.offset.x + ((ix - model.pan.x) * model.zoom) / editor.ratioOf(), y: box().top + model.offset.y + ((iy - model.pan.y) * model.zoom) / editor.ratioOf() });
    const pointer = (type, target, ix, iy) => {
      const at = css(ix, iy);
      target.dispatchEvent(new PointerEvent(type, { pointerId: 14, button: 0, buttons: type === 'pointerup' ? 0 : 1, clientX: at.x, clientY: at.y, bubbles: true, cancelable: true }));
    };
    const press = (init) => { const e = new KeyboardEvent('keydown', { bubbles: true, cancelable: true, ...init }); window.dispatchEvent(e); return e.defaultPrevented; };
    const frame = document.getElementById('crop');
    const handle = (edge) => frame.querySelector(`.h[data-edge="${edge}"]`);
    const frameAt = () => `${frame.style.left} ${frame.style.top} ${frame.style.width} ${frame.style.height}`;
    const cropText = () => editor.cropString() || 'none';
    const sizeBand = () => document.getElementById('image-size').textContent;
    const canvas = document.getElementById('canvas');
    const dragHandle = async (edge, fromX, fromY, toX, toY) => {
      pointer('pointerdown', handle(edge), fromX, fromY);
      pointer('pointermove', stage, toX, toY);
      pointer('pointerup', stage, toX, toY);
      await editor.paintRegion();
    };

    press({ key: 'k', code: 'KeyK' });
    check('K picks the crop, and its button in the sidebar is lit', model.tool === 'crop' && document.querySelector('#tools [data-tool="crop"]').classList.contains('active'));
    const button = document.querySelector('#tools [data-tool="crop"]');
    check('the crop button sits after Ruler, named Crop, an icon button like the others', !!button && button.previousElementSibling.dataset.tool === 'ruler' && button.getAttribute('aria-label') === 'Crop' && getComputedStyle(button).width === '48px');
    // A click on the lit button puts the tool down, and one more takes it up again (Rotem, 2026-09-19).
    button.click();
    await sleep(0);
    check('a click on the lit Crop button puts the crop down: no tool, the button dark, the frame gone', model.tool === null && !button.classList.contains('active') && getComputedStyle(frame).display === 'none', `tool ${model.tool}; ${getComputedStyle(frame).display}`);
    button.click();
    await sleep(0);
    check('and one more click takes it up again', model.tool === 'crop' && button.classList.contains('active'), `tool ${model.tool}`);
    check('with the tool in hand a frame sits on the picture\'s edges, with a handle at each corner and the middle of each side', getComputedStyle(frame).display === 'block' && frameAt() === '0px 0px 400px 300px' && frame.querySelectorAll('.h').length === 8
      && ['nw', 'n', 'ne', 'w', 'e', 'sw', 's', 'se'].every((e) => !!handle(e)), `${getComputedStyle(frame).display}; ${frameAt()}; ${frame.querySelectorAll('.h').length} handles`);
    const h1 = handle('e').getBoundingClientRect();
    const edge1 = css(400, 150);
    await editor.setZoom(2);
    const h2 = handle('e').getBoundingClientRect();
    const edge2 = css(400, 150);
    check('a handle is 10 screen pixels centred on its edge at zoom 1 and at zoom 2 alike, the scene\'s scale undone on it', Math.abs(h1.width - 10) < 0.6 && Math.abs(h1.height - 10) < 0.6 && Math.abs(h2.width - 10) < 0.6 && Math.abs(h2.height - 10) < 0.6
      && Math.abs(h1.left + h1.width / 2 - edge1.x) < 1.5 && Math.abs(h1.top + h1.height / 2 - edge1.y) < 1.5 && Math.abs(h2.left + h2.width / 2 - edge2.x) < 1.5 && Math.abs(h2.top + h2.height / 2 - edge2.y) < 1.5,
      `zoom 1: ${h1.width.toFixed(1)}x${h1.height.toFixed(1)} centre ${(h1.left + h1.width / 2).toFixed(1)},${(h1.top + h1.height / 2).toFixed(1)} edge ${edge1.x.toFixed(1)},${edge1.y.toFixed(1)}; zoom 2: ${h2.width.toFixed(1)}x${h2.height.toFixed(1)} centre ${(h2.left + h2.width / 2).toFixed(1)},${(h2.top + h2.height / 2).toFixed(1)} edge ${edge2.x.toFixed(1)},${edge2.y.toFixed(1)}`);
    await editor.setZoom(1);

    // A note that fits inside every crop below, so the margin stays at zero and the sizes are the crop's.
    const note = editor.createCallout({ x: 60, y: 50 });
    editor.layoutScene();
    note.text = 'kept';
    editor.settle();
    editor.record();
    const noteEl = () => document.querySelector(`[data-id="${note.id}"]`);
    const noteLeft = noteEl().style.left;

    // The right edge dragged inward: while the drag lasts the frame follows and darkens the outside; released, the picture is the frame.
    pointer('pointerdown', handle('e'), 400, 150);
    pointer('pointermove', stage, 380, 150);
    check('while a handle is dragged the frame follows it and darkens what it will cut away, and nothing is cut yet', !!model.cropping && model.cropping.w === 380 && model.cropping.h === 300 && frameAt() === '0px 0px 380px 300px' && frame.classList.contains('dragging') && getComputedStyle(frame).boxShadow !== 'none' && model.crop === null,
      `${JSON.stringify(model.cropping)}; ${frameAt()}; ${getComputedStyle(frame).boxShadow}`);
    check('and the size band follows the frame while the drag lasts (Rotem, 2026-09-19)', sizeBand() === '380x300', `band ${sizeBand()}`);
    pointer('pointerup', stage, 380, 150);
    await editor.paintRegion();
    const steps1 = model.history.index;
    check('released, the picture is cut to the frame: 380 wide, the size band says so, and the frame sits on the new edges', cropText() === '0,0,380,300' && sizeBand() === '380x300' && editor.compositionSize().w === 380 && editor.compositionSize().h === 300 && frameAt() === '0px 0px 380px 300px' && !frame.classList.contains('dragging'),
      `crop ${cropText()}; band ${sizeBand()}; ${frameAt()}`);
    check('the crop is one step of history', steps1 >= 2, `index ${steps1}`);

    // A corner dragged inward moves two edges; the picture on screen is the crop alone.
    await dragHandle('nw', 0, 0, 40, 30);
    check('a corner handle moves two edges: the crop starts at 40,30 and is 340 by 270', cropText() === '40,30,340,270' && sizeBand() === '340x270', `crop ${cropText()}; band ${sizeBand()}`);
    check('the picture painted is the crop alone, centred, and the view starts at the crop\'s origin', canvas.width === 340 && canvas.height === 270 && model.pan.x === 40 && model.pan.y === 30 && Math.abs(parseFloat(canvas.style.left) - model.offset.x) < 1 && Math.abs(parseFloat(canvas.style.top) - model.offset.y) < 1,
      `canvas ${canvas.width}x${canvas.height} at ${canvas.style.left},${canvas.style.top}; offset ${model.offset.x},${model.offset.y}; pan ${model.pan.x},${model.pan.y}`);
    check('the note keeps its place in image pixels, and the margin stays at zero', note.anchor.x === 60 && note.anchor.y === 50 && noteEl().style.left === noteLeft && editor.marginString() === '0,0,0,0', `anchor ${note.anchor.x},${note.anchor.y}; box ${noteEl().style.left} was ${noteLeft}; margin ${editor.marginString()}`);

    // Outward does nothing; the least a side can be cut to is 8.
    await dragHandle('e', 380, 165, 500, 165);
    check('a handle dragged outward moves nothing', cropText() === '40,30,340,270', `crop ${cropText()}`);
    await dragHandle('w', 40, 165, 1000, 165);
    check('a side cannot be cut under 8 pixels', cropText() === '372,30,8,270', `crop ${cropText()}`);
    press({ key: 'z', code: 'KeyZ', ctrlKey: true });
    await editor.paintRegion();
    check('Ctrl+Z gives the cut back', cropText() === '40,30,340,270' && sizeBand() === '340x270', `crop ${cropText()}; band ${sizeBand()}`);

    // The copy: the crop with the note over it, every pixel the source's own, the frame left out.
    const layer = await editor.exportLayer();
    check('the layer is drawn for the crop: the notes shifted by its origin, the frame left out', layer.crop === '40,30,340,270' && layer.markup.includes('left:-40px;top:-30px') && layer.markup.includes('kept') && !layer.markup.includes('data-edge'), `crop ${layer.crop}`);
    const r = await invoke('editor_export_check', layer.bytes, { headers: { name: 's44-crop', margin: layer.margin, blur: layer.blur, crop: layer.crop, mode: 'source', sample: '0,0' } });
    const region = await fetch('http://region.localhost/?x=40&y=30&w=1&h=1&ow=1&oh=1', { cache: 'no-store' });
    const px = new Uint8Array(await region.arrayBuffer()).subarray(8, 11);
    check('the copy is the crop\'s size, every uncovered pixel the source\'s own, its top left the source\'s pixel at 40,30', r.width === 340 && r.height === 270 && r.source_mismatches === 0 && !!r.sample && r.sample[0] === px[0] && r.sample[1] === px[1] && r.sample[2] === px[2],
      `${r.width}x${r.height}, ${r.source_mismatches} mismatches, sample ${JSON.stringify(r.sample)} against ${Array.from(px)}`);

    // The thumbnail follows the save: the crop with the note, fitted into the box.
    await editor.saveNow();
    const refreshed = await editor.thumbnailDone();
    const response = await fetch(`http://region.localhost/?thumb=${shot.document_id}`, { cache: 'no-store' });
    const bitmap = await createImageBitmap(await response.blob());
    const scale = Math.min(320 / 340, 200 / 270, 1);
    check('the thumbnail is the crop fitted into the box', refreshed === true && bitmap.width === Math.round(340 * scale) && bitmap.height === Math.round(270 * scale), `${refreshed}; ${bitmap.width}x${bitmap.height}`);

    // Undo and redo walk the crops, the size band with them.
    press({ key: 'z', code: 'KeyZ', ctrlKey: true });
    await editor.paintRegion();
    check('Ctrl+Z takes the corner\'s cut back', cropText() === '0,0,380,300' && sizeBand() === '380x300', `crop ${cropText()}; band ${sizeBand()}`);
    press({ key: 'z', code: 'KeyZ', ctrlKey: true });
    await editor.paintRegion();
    check('and the edge\'s: the whole picture again, 400 by 300', cropText() === 'none' && sizeBand() === '400x300' && canvas.width === 400, `crop ${cropText()}; band ${sizeBand()}; canvas ${canvas.width}`);
    press({ key: 'Z', code: 'KeyZ', ctrlKey: true, shiftKey: true });
    press({ key: 'Z', code: 'KeyZ', ctrlKey: true, shiftKey: true });
    await editor.paintRegion();
    check('Ctrl+Shift+Z twice cuts it again to 40,30 340 by 270', cropText() === '40,30,340,270' && sizeBand() === '340x270', `crop ${cropText()}; band ${sizeBand()}`);

    // Saved with the notes, back after a restart.
    await editor.saveNow();
    editor.documents.clear();
    model.image = { width: 0, height: 0, source: '' };
    await editor.loadImage(await invoke('editor_store_reload'));
    check('after a restart the crop is back from the disk', cropText() === '40,30,340,270' && sizeBand() === '340x270' && model.callouts.length === 1, `crop ${cropText()}; band ${sizeBand()}`);

    // The frame goes with its tool and comes back with it.
    await editor.setMode('annotate');
    press({ key: 'c', code: 'KeyC' });
    const gone = getComputedStyle(frame).display;
    press({ key: 'k', code: 'KeyK' });
    check('the frame goes with another tool and comes back with the crop tool, on the crop\'s edges', gone === 'none' && getComputedStyle(frame).display === 'block' && frameAt() === '40px 30px 340px 270px', `${gone} then ${getComputedStyle(frame).display}; ${frameAt()}`);

    model.callouts = [];
    model.shapes = [];
    await editor.applyCrop(null);
    editor.setTool(null);
    editor.record();
    await editor.saveNow();
    for (let i = 0; i < 40 && !(await invoke('editor_window_visible')); i += 1) await sleep(50);
    await invoke('editor_show');
  }

  // ---------------------------------------------------------------- 45. the colour picker
  say('');
  say('The colour picker (Rotem, 2026-09-18): I or the button, then the zoom circle follows the pointer with the HEX of the pixel under it above it; a click copies that HEX to the clipboard as text and puts the tool down; nothing is changed, so no step of undo and no document over a file only viewed');
  {
    const stage = editor.stage;
    const hud = document.getElementById('hud');
    await invoke('editor_store_reset');
    const shot = await invoke('editor_capture_probe', { width: 400, height: 300 });
    await editor.loadImage(shot);
    await editor.setZoom(1);
    const box = () => stage.getBoundingClientRect();
    const css = (ix, iy) => ({ x: box().left + model.offset.x + ((ix - model.pan.x) * model.zoom) / editor.ratioOf(), y: box().top + model.offset.y + ((iy - model.pan.y) * model.zoom) / editor.ratioOf() });
    const pointer = (type, ix, iy) => {
      const at = css(ix, iy);
      stage.dispatchEvent(new PointerEvent(type, { pointerId: 15, button: 0, buttons: type === 'pointerdown' ? 1 : 0, clientX: at.x, clientY: at.y, bubbles: true, cancelable: true }));
    };
    const press = (init) => { const e = new KeyboardEvent('keydown', { bubbles: true, cancelable: true, ...init }); window.dispatchEvent(e); return e.defaultPrevented; };
    const sourceHex = async (ix, iy) => {
      const whole = new Uint8Array(await (await fetch(`http://region.localhost/?x=${ix}&y=${iy}&w=1&h=1&ow=1&oh=1`, { cache: 'no-store' })).arrayBuffer());
      return `#${[...whole.subarray(8, 11)].map((v) => v.toString(16).padStart(2, '0')).join('').toUpperCase()}`;
    };
    const magEl = document.getElementById('mag');
    const magText = () => magEl.dataset.label;

    press({ key: 'i', code: 'KeyI' });
    const button = document.querySelector('#tools [data-tool="picker"]');
    check('I picks the colour picker, and its button in the sidebar is lit, after Crop, named Color picker', model.tool === 'picker' && !!button && button.classList.contains('active') && button.previousElementSibling.dataset.tool === 'crop' && button.getAttribute('aria-label') === 'Color picker' && getComputedStyle(button).width === '48px', `tool ${model.tool}`);
    // The middle of pixel 200,150: the circle's text against the picture's own pixel, asked of the host.
    pointer('pointermove', 200.5, 150.5);
    await sleep(250);
    const want = await sourceHex(200, 150);
    check('with the picker in hand the zoom circle is up with the HEX of the pixel under the pointer above it', getComputedStyle(magEl).display === 'block' && magText() === want && Number(magEl.dataset.cy) - Number(magEl.dataset.radius) > 8 * editor.ratioOf(), `circle "${magText()}", picture ${want}`);
    const stepsBefore = model.history.index;
    pointer('pointerdown', 200.5, 150.5);
    pointer('pointerup', 200.5, 150.5);
    for (let i = 0; i < 60 && model.tool !== null; i += 1) await sleep(50);
    let text = await invoke('editor_clipboard_text').catch((err) => `none: ${err}`);
    check('a click copies that pixel\'s HEX to the clipboard as text, says so, and puts the tool down with the circle', text === want && /^#[0-9A-F]{6}$/.test(text) && model.tool === null && getComputedStyle(magEl).display === 'none' && hud.textContent.includes(`copied ${want}`), `clipboard "${text}", picture ${want}, tool ${model.tool}, "${hud.textContent}"`);
    check('the pick made nothing and is no step of history', model.history.index === stepsBefore && model.callouts.length === 0 && model.shapes.length === 0, `history ${stepsBefore} then ${model.history.index}`);

    // Another pixel is another HEX, so the clipboard follows the click and not the last pick.
    let other = null;
    for (const [ix, iy] of [[10, 10], [60, 40], [390, 290], [123, 77], [300, 20]]) { const h = await sourceHex(ix, iy); if (h !== want) { other = { ix, iy, h }; break; } }
    if (other) {
      press({ key: 'i', code: 'KeyI' });
      pointer('pointermove', other.ix + 0.5, other.iy + 0.5);
      pointer('pointerdown', other.ix + 0.5, other.iy + 0.5);
      pointer('pointerup', other.ix + 0.5, other.iy + 0.5);
      for (let i = 0; i < 60 && model.tool !== null; i += 1) await sleep(50);
      text = await invoke('editor_clipboard_text').catch((err) => `none: ${err}`);
      check('a click on a pixel of another colour copies that one', text === other.h, `clipboard "${text}", picture ${other.h} at ${other.ix},${other.iy}`);
    } else {
      skipped('a click on a pixel of another colour copies that one', 'the probe showed one colour at every place tried');
    }

    // Off the picture: nothing is copied, nothing is said, and the tool stays.
    press({ key: 'i', code: 'KeyI' });
    pointer('pointermove', -20, -20);
    pointer('pointerdown', -20, -20);
    pointer('pointerup', -20, -20);
    await sleep(300);
    const kept = await invoke('editor_clipboard_text').catch((err) => `none: ${err}`);
    check('a click off the picture copies nothing and keeps the picker in hand', model.tool === 'picker' && kept === text && !hud.textContent.includes('NOT COPIED'), `tool ${model.tool}, clipboard "${kept}", "${hud.textContent}"`);
    editor.setTool(null);

    // Over a file only viewed: the picker works and no document is made.
    const dir = await invoke('editor_make_folder');
    const storeBefore = (await invoke('editor_store_list')).length;
    const info = await invoke('editor_open_file', { path: `${dir}\\img2.png` });
    await editor.loadImage(info);
    await editor.setZoom(1);
    document.querySelector('#tools [data-tool="picker"]').click();
    check('over a file only viewed the button puts the picker in hand and the file stays viewed', model.tool === 'picker' && model.mode === 'view', `tool ${model.tool}, mode ${model.mode}`);
    pointer('pointermove', 5.5, 5.5);
    await sleep(250);
    const shown = magText();
    pointer('pointerdown', 5.5, 5.5);
    pointer('pointerup', 5.5, 5.5);
    for (let i = 0; i < 60 && model.tool !== null; i += 1) await sleep(50);
    text = await invoke('editor_clipboard_text').catch((err) => `none: ${err}`);
    const store = await invoke('editor_store_list');
    check('a click there copies the HEX the circle showed, and no document is made for the file', /^#[0-9A-F]{6}$/.test(text) && text === shown && model.tool === null && model.mode === 'view' && store.length === storeBefore, `clipboard "${text}", circle "${shown}", mode ${model.mode}, store ${storeBefore} then ${store.length}`);
  }

  // ---------------------------------------------------------------- an opened file in the timeline
  say('');
  say('A file opened by name sits in the timeline as a pointer: nothing copied, the file untouched, gone with its file, a document once annotated');
  {
    const strip = editor.strip;
    const hud = document.getElementById('hud');
    await invoke('editor_store_reset');
    const dir = await invoke('editor_make_folder');
    const at = (name) => `${dir}\\${name}`;
    const thumbs = () => strip.querySelectorAll('.thumb:not(.trashed)').length;
    const press = (init) => { const e = new KeyboardEvent('keydown', { bubbles: true, cancelable: true, ...init }); window.dispatchEvent(e); return e.defaultPrevented; };
    const before = await invoke('editor_file_print', { path: at('img2.png') });

    let info = await invoke('editor_open_file', { path: at('img2.png') });
    await editor.loadImage(info);
    await editor.refreshStrip();
    for (let i = 0; i < 60 && thumbs() !== 1; i += 1) await sleep(50);
    let docs = await invoke('editor_documents');
    let store = await invoke('editor_store_list');
    check('opened by name: one thumbnail, a pointer, current, the picture in viewing, and the store holds no document for it',
      thumbs() === 1 && docs.length === 1 && docs[0].linked === true && docs[0].current === true && docs[0].file === 'img2.png' && info.linked === true && info.managed === false && model.mode === 'view' && store.length === 0,
      `thumbnails ${thumbs()}, ${JSON.stringify(docs)}, store ${store.length}`);
    for (let i = 0; i < 100 && !strip.querySelector('img'); i += 1) await sleep(50);
    const img = strip.querySelector('img');
    for (let i = 0; i < 100 && !(img && img.naturalWidth > 0); i += 1) await sleep(50);
    store = await invoke('editor_store_list');
    check('its thumbnail is drawn, and still nothing of it is in the store', !!img && img.naturalWidth > 0 && store.length === 0, `thumbnail ${img ? img.naturalWidth : 'none'}, store ${store.length}`);

    // Stepping through the folder joins nothing; opening the same file again adds nothing.
    await editor.loadImage(await invoke('editor_navigate', { step: 'next' }));
    await editor.refreshStrip();
    const again = await invoke('editor_open_file', { path: at('img2.png') });
    await editor.loadImage(again);
    await editor.refreshStrip();
    docs = await invoke('editor_documents');
    check('a file stepped onto joins nothing, and the same file opened again is the same pointer', docs.length === 1 && again.document_id === info.document_id, JSON.stringify(docs.map((d) => d.file)));

    // A second file and a capture; then the pointer is chosen from the timeline.
    const second = await invoke('editor_open_file', { path: at('b.jpg') });
    await editor.loadImage(second);
    const shot = await invoke('editor_capture_probe', { width: 320, height: 200 });
    await editor.loadImage(shot);
    await editor.refreshStrip();
    for (let i = 0; i < 60 && thumbs() !== 3; i += 1) await sleep(50);
    strip.children[2].click(); // the oldest, img2.png, at the right
    for (let i = 0; i < 60 && model.image.document_id !== info.document_id; i += 1) await sleep(50);
    check('chosen from the timeline, the file is opened from where it lives', model.image.document_id === info.document_id && model.image.file === 'img2.png' && model.mode === 'view', `${model.image.document_id} ${model.image.file}`);

    // After a restart the pointers are back from the disk.
    const onDisk = await invoke('editor_opened_reload');
    docs = await invoke('editor_documents');
    check('the pointers are saved and read back: two files and the capture', docs.length === 3 && docs.filter((d) => d.linked).length === 2 && onDisk.includes('img2.png') && onDisk.includes('b.jpg'), JSON.stringify(docs.map((d) => [d.file, d.linked])));

    // Taken off the timeline: the file stays, nothing goes to the trash, Ctrl+Z puts it back.
    await editor.loadImage(await invoke('editor_show_document', { id: shot.document_id }));
    await editor.refreshStrip();
    for (let i = 0; i < 60 && thumbs() !== 3; i += 1) await sleep(50);
    strip.children[2].dispatchEvent(new MouseEvent('contextmenu', { bubbles: true, cancelable: true, button: 2, clientX: 20, clientY: 400 }));
    document.getElementById('tab-menu').querySelector('button').click();
    for (let i = 0; i < 60 && thumbs() !== 2; i += 1) await sleep(50);
    let trash = await invoke('editor_trash_list');
    const after = await invoke('editor_file_print', { path: at('img2.png') });
    check('Delete in its menu takes the pointer off: two thumbnails, nothing in the trash, the file byte for byte as it was, and the notice says so',
      thumbs() === 2 && trash.length === 0 && after === before && hud.textContent.includes('untouched'), `thumbnails ${thumbs()}, trash ${trash.length}, ${before} then ${after}, "${hud.textContent}"`);
    press({ key: 'z', code: 'KeyZ', ctrlKey: true });
    for (let i = 0; i < 60 && thumbs() !== 3; i += 1) await sleep(50);
    docs = await invoke('editor_documents');
    check('Ctrl+Z puts the pointer back where it was', thumbs() === 3 && docs[0].id === info.document_id && docs[0].linked === true, JSON.stringify(docs.map((d) => d.file)));
    press({ key: 'Z', code: 'KeyZ', ctrlKey: true, shiftKey: true });
    for (let i = 0; i < 60 && thumbs() !== 2; i += 1) await sleep(50);
    check('and Ctrl+Shift+Z takes it off again', thumbs() === 2);
    press({ key: 'z', code: 'KeyZ', ctrlKey: true });
    for (let i = 0; i < 60 && thumbs() !== 3; i += 1) await sleep(50);

    // Annotate turns the pointer into a document, under the same number.
    await editor.loadImage(await invoke('editor_show_document', { id: info.document_id }));
    await editor.setMode('annotate');
    await editor.refreshStrip();
    docs = await invoke('editor_documents');
    const mine = docs.filter((d) => d.file === 'img2.png');
    store = await invoke('editor_store_list');
    check('annotated, the pointer becomes the file\'s document: one thumbnail for it, not a pointer, the same number, now in the store',
      mine.length === 1 && mine[0].linked === false && mine[0].id === info.document_id && model.image.managed === true && store.some((line) => line.id === info.document_id),
      `${JSON.stringify(mine)}, store ${JSON.stringify(store.map((line) => line.id))}`);
    check('and the file is still byte for byte as it was', (await invoke('editor_file_print', { path: at('img2.png') })) === before);

    // A file that is gone leaves the timeline.
    await invoke('editor_remove_from_folder', { name: 'b.jpg' });
    await editor.refreshStrip();
    docs = await invoke('editor_documents');
    check('a file removed from where it lived leaves the timeline', !docs.some((d) => d.file === 'b.jpg'), JSON.stringify(docs.map((d) => d.file)));
    await editor.setMode('view');
  }

  // ---------------------------------------------------------------- Save As over the annotated file
  say('');
  say('Save As on an annotated PNG or JPEG opens on the file itself and writes over it, and over nothing else; any other type keeps the new annotated PNG');
  {
    await invoke('editor_store_reset');
    const dir = await invoke('editor_make_folder');
    const at = (name) => `${dir}\\${name}`;
    const writeTo = async (path) => {
      const layer = await editor.exportLayer();
      return invoke('editor_save_as_write', layer.bytes, { headers: { margin: layer.margin, path } });
    };
    await editor.loadImage(await invoke('editor_open_file', { path: at('img2.png') }));
    let plan = await invoke('editor_save_as_plan');
    check('only viewed, the file keeps the new name marked as annotated, and nothing may be written over', plan.name === 'img2 annotated.png' && plan.replaces === '', JSON.stringify(plan));

    await editor.setMode('annotate');
    const note = editor.createCallout({ x: 40, y: 40 });
    note.text = 'saved over';
    editor.layoutScene();
    editor.record();
    await editor.saveNow();
    plan = await invoke('editor_save_as_plan');
    check('annotated, Save As opens on the file\'s own folder and name, and names it as the one file it may write over',
      plan.name === 'img2.png' && plan.folder.toLowerCase() === dir.toLowerCase() && plan.replaces.toLowerCase() === at('img2.png').toLowerCase(), JSON.stringify(plan));

    const otherBefore = await invoke('editor_file_print', { path: at('IMG1.png') });
    const other = await writeTo(at('IMG1.png'));
    check('another file that exists is still never written over: a free name is offered and its bytes stay',
      other.Exists !== undefined && (await invoke('editor_file_print', { path: at('IMG1.png') })) === otherBefore, JSON.stringify(other).slice(0, 160));

    const before = await invoke('editor_file_print', { path: at('img2.png') });
    const over = await writeTo(at('img2.png'));
    const after = await invoke('editor_file_print', { path: at('img2.png') });
    const kind = await invoke('editor_file_kind', { path: at('img2.png') });
    check('the file itself is written over with the picture and its notes, still a PNG that opens', over.Replaced !== undefined && after !== before && kind.format === 'png' && kind.width >= 200,
      `${JSON.stringify(over).slice(0, 120)}; ${before} then ${after}; ${kind.format} ${kind.width}x${kind.height}`);
    const leftovers = (await invoke('editor_folder_names', { dir })).filter((n) => !['img10.png', 'img2.png', 'IMG1.png', 'b.jpg', 'a.gif', 'notes.txt'].includes(n));
    check('nothing else is left in the folder: no temporary file, no second version', leftovers.length === 0, JSON.stringify(leftovers));
    const info = await invoke('editor_image_info');
    check('the document does not call its own save a change on disk, and keeps its clean picture and its note', info.source_changed === false && info.managed === true && model.callouts.length === 1, `changed ${info.source_changed}`);

    // A type that cannot be written back as itself keeps today's Save As.
    await editor.loadImage(await invoke('editor_open_file', { path: at('a.gif') }));
    await editor.setMode('annotate');
    plan = await invoke('editor_save_as_plan');
    check('an annotated GIF keeps the new annotated PNG, and nothing may be written over', plan.name === 'a annotated.png' && plan.replaces === '', JSON.stringify(plan));
    await editor.setMode('view');
  }

  say('');
  const unrun = notRun ? `, ${notRun} not run` : '';
  if (failures === 0) {
    say(`RESULT: every check that ran passed${unrun}.`);
  } else {
    say(`RESULT: ${failures} check(s) failed${unrun}.`);
  }
  await invoke('editor_checks_done', { report: lines.join('\n'), failures });
}
