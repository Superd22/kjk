"""Writes the Zones section into template.html (replaces everything between the ZONES markers, or inserts it before the first step) and fills the placeholders into likeness-hq.html."""
import json, re, numpy as np
D = '/Users/david/.claude/tools/likeness-fit-data/kotor-heads/'
d = json.load(open(D + 'kotor_fit_v6.json'))
C, F = 'FLAME + openings', 'FLAME'
def st(z, s):
    a = np.array([r[s][z] for r in d]); p = np.array([r[s][z + ' p95'] for r in d])
    w = sorted(zip(a, [r['head'] for r in d]))[-3:][::-1]
    return a.mean(), p.mean(), int((a <= .75).sum()), w
def pr(k, s):
    a = np.array([r[s]['proportions'][k] for r in d if k in r[s]['proportions']])
    return a.mean(), abs(a).mean(), int((a > 0).sum()), len(a)
def n(x): return '<span class="num">%.2f mm</span>' % x
def pro(k, label, unit_note=''):
    m, ab, lon, c = pr(k, C)
    side = 'longer' if m >= 0 else 'shorter'
    return ('<li>On average our %s is %s %s than KOTOR\'s. The typical head differs by %s, in either direction. Our %s is larger on %d of the %d heads that have this measure.</li>'
            % (label, n(abs(m)), side, n(ab), label, lon, c))

Z = [
 ('forehead', 'Forehead', 'weak', 'Not reproduced', 'The forehead is the one zone that is far from the pass mark. KOTOR models hair as part of the skull, so the forehead\'s score includes hair that our bare head does not have.', None,
  ['A test of step 4 holds our upper skull in a band just under KOTOR\'s outer surface, hair included. On the older targets it brought FLAME\'s skull from <span class="num">2.42 mm</span> off to <span class="num">0.32 mm</span>.',
   'Three of the 57 heads are within the zone mark, against one with FLAME alone.'],
  ['It is not part of the fit yet, and it has not run on the current targets.',
   'On the older targets, the face\'s score moved from <span class="num">0.68 mm</span> to <span class="num">0.74 mm</span> when the skull was held.',
   'Kreia, <code>mod_hp_pfha01</code> and <code>mod_hp_pfhb02</code> have hair panels and no skull.'],
  ['Merge the skull hold into the fit as an option, with a switch for each head.',
   'Switch it off for Kreia and <code>mod_hp_pfhb02</code>.',
   'Run it on the 57 heads and read the forehead score again.'], None),
 ('brows', 'Brows', 'part', 'Close', 'The brows are about <span class="num">0.06 mm</span> over the zone mark on average. Fewer than half of the heads are within it.', None,
  ['The eye opening controls moved the number of heads within the mark from 19 to 23.'],
  ['The brows are the worst of the zones that are close. 34 of the 57 heads are over the mark, and the worst head is <code>pmhc04</code>.',
   'On the feature sheet, the fit slid the brow peak vertex up to <span class="num">2.9 mm</span> along the surface on Carth, which moves the paint with it.'],
  ['After step 4, learn new shapes from what FLAME leaves over on the KOTOR heads, which is about <span class="num">0.6 mm</span>. Test each learned shape on heads it was not learned from.'], 'pic_brows'),
 ('eyes', 'Around the eyes', 'good', 'Reproduced', 'The shape around the eyes now passes the zone mark on 51 of the 57 heads. This verdict is about the shape only. The paint on the closing lid and the lash strip are still missing, as the lists below say.', ['eye centres apart'],
  ['Each eye has 26 new controls for the opening\'s width, its place sideways, and the height and depth of the lid edge at seven stops along the lids.',
   'Our lid edge now lies <span class="num">0.87 mm</span> from KOTOR\'s slit on average, against <span class="num">5.85 mm</span> with FLAME alone. The opening\'s height is <span class="num">0.07 mm</span> off.',
   'The blink is computed on each head from its own lid edges. All 57 heads close with less than <span class="num">1 mm</span> left between the lids.'],
  ['A closing lid shows the liner and lash paint taken from the open lid, so it looks smeared and dark. This is worst on Bastila, <code>pfha02</code> and Meetra.',
   'The lash strip does not exist yet. Bastila\'s and Kreia\'s lower liner shows doubled, and Bastila\'s liner wing cannot be made.',
   'The inner corner\'s depth controls hit their limit on 31 to 36 of the 57 heads. The sideways controls hit their limit on 10 to 13 heads.',
   'The opening is still about <span class="num">1 mm</span> narrow on average, because both corners stop about 1 mm short of KOTOR\'s.',
   'On <code>mod_hp_pfha01</code> our lid edge lies <span class="num">7.18 mm</span> from KOTOR\'s slit. The next worst head is <span class="num">2.1 mm</span> off, so this target may be faulty.',
   'A small dent shows above the outer corner when the eye closes.'],
  ['Paint the lid skin with plain skin from above the eye.',
   'Build the lash strip, a thin strip on the lid edge that carries the lashes and the liner, with its own sliders outside FLAME.',
   'Widen the ranges of the depth and sideways controls, and check the target of <code>mod_hp_pfha01</code>.',
   'Fix the dent above the outer corner.'], 'pic_eyes'),
 ('nose', 'Nose', 'good', 'Reproduced', 'The nose passes the zone mark on 56 of the 57 heads.', ['nose width', 'nose height', 'nose length'],
  ['The nose is within the zone mark on 56 of the 57 heads, with FLAME alone on 55.',
   'The nose width is within about <span class="num">0.3 mm</span> on a typical head.'],
  ['Our nose is shorter than KOTOR\'s on average, and its height is off by about <span class="num">0.5 mm</span> for a typical head, as the proportions above show.',
   'The worst head, <code>pmhc04</code>, sits right at the mark of <span class="num">0.75 mm</span>.'],
  ['No nose-specific work is planned. The nose is checked again when the shapes learned in step 5 are tested.'], 'pic_nose'),
 ('cheeks', 'Cheeks', 'part', 'Close', 'The cheeks are about <span class="num">0.02 mm</span> over the zone mark on average, and the eye opening controls did not help them.', ['cheekbone width'],
  ['The cheekbone width is right on average. The mean difference is <span class="num">0.00 mm</span>.'],
  ['Only 19 of the 57 heads are within the mark, against 23 with FLAME alone. The eye opening controls moved the cheeks a little the wrong way.',
   'The typical cheekbone width differs by about <span class="num">0.5 mm</span>, in either direction.'],
  ['After step 4, learn new shapes from what FLAME leaves over on the KOTOR heads, and test them on heads they were not learned from.'], None),
 ('mouth', 'Mouth', 'good', 'Reproduced', 'The mouth is the best zone. It passes on 56 of the 57 heads.', ['mouth width', 'upper lip height'],
  ['The mouth is within the zone mark on 56 of the 57 heads, and on all 57 with FLAME alone.',
   'The mouth width is right on average and within about <span class="num">0.3 mm</span> on a typical head.'],
  ['Our upper lip is taller than KOTOR\'s on 35 of the 57 heads, and a typical head differs by <span class="num">0.50 mm</span>.',
   'With the eye opening controls, one head (<code>pmhc04</code>) sits at the mark of <span class="num">0.75 mm</span>.'],
  ['No mouth-specific work is planned. The mouth is checked again when the shapes learned in step 5 are tested.'], 'pic_mouth'),
 ('chin', 'Chin', 'part', 'Close', 'The chin is about <span class="num">0.03 mm</span> over the zone mark on average. Fewer than half of the heads are within it.', ['nose base to chin'],
  ['The eye opening controls moved the number of heads within the mark from 18 to 21.',
   ],
  ['FLAME\'s lower face comes out a little bigger than KOTOR\'s. Our chin lies lower than KOTOR\'s, measured from the base of the nose, as shown above.',
   'The worst heads are Jolee, Kenobi and <code>pmhc04</code>.'],
  ['After step 4, learn new shapes from what FLAME leaves over on the KOTOR heads, and test them on heads they were not learned from.'], 'pic_chin'),
 ('jaw', 'Jaw', 'part', 'Close', 'The jaw is about <span class="num">0.05 mm</span> over the zone mark on average. Fewer than half of the heads are within it.', ['jaw width', 'face height'],
  ['The eye opening controls moved the number of heads within the mark from 10 to 18.',
   'Step 5 quotes an earlier fit, in which the jaw was <span class="num">0.8 mm</span> wider than KOTOR\'s. In this fit the jaw is <span class="num">0.44 mm</span> wider, and <span class="num">0.53 mm</span> wider with FLAME alone.'],
  ['The jaw is still the zone with one of the largest proportion differences. Its width differs by <span class="num">0.70 mm</span> for a typical head.'],
  ['After step 4, learn new shapes from what FLAME leaves over on the KOTOR heads, and test them on heads they were not learned from.',
   'Check that each head lies closer to its own KOTOR head than to any other.'], 'pic_jaw'),
]
PIC = {'pic_brows': ('Carth, with brows and eyes cut from the feature sheet', 'Carth'),
       'pic_eyes': ('Bastila', 'Bastila'), 'pic_nose': ('Bastila', 'Bastila'), 'pic_mouth': ('Bastila', 'Bastila'),
       'pic_chin': ('Bastila', 'Bastila'), 'pic_jaw': ('The player head <code>pmhc01</code>', 'pmhc01')}
LABEL = {'forehead':'forehead','brows':'brows','around the eyes':'zone','nose':'nose','cheeks':'cheeks','mouth':'mouth','chin':'chin','jaw':'jaw'}

out = []
pnum = 1
links = ' '.join('<a href="#zone-%s">%s</a>' % (k, t.lower()) for k, t, *_ in Z)
out.append('''  <!--ZONES-->
  <section class="block" id="zones-detail" aria-labelledby="zd-title">
    <h2 id="zd-title">Zones</h2>
    <p>This section has one part for each zone of the face. Each part gives a verdict on one question. Do our sliders reproduce this zone on the KOTOR heads? A zone is reproduced when its mean distance is within the mark of <span class="num">0.75 mm</span>, and when most of the 57 heads are within it too. A zone is close when its mean is within <span class="num">0.15 mm</span> of the mark. Otherwise it is not reproduced.</p>
    <p>The numbers are the mean and the 95th percentile over the 57 heads, as in the chart above. The first value is for FLAME with the eye opening controls, which is the current state. The second value is for FLAME alone. A proportion is how much longer a part is on our head than on KOTOR's, in millimetres. A negative value means ours is shorter.</p>
    <p>Jump to: ''' + links + '''.</p>
  </section>
''')
for key, title, cls, verdict, intro, props, done, dislike, nxt, pic in Z:
    zk = 'around the eyes' if key == 'eyes' else key
    a = st(zk, C); b = st(zk, F)
    worst = ', '.join('<code>%s</code> <span class="num">%.2f</span>' % (h, x) for x, h in a[3])
    out.append('''  <section class="zone" id="zone-%s">
    <div class="step-head"><h2>%s</h2><span class="pill %s">%s</span></div>
    <p>%s</p>
    <div class="zstats">
      <div><span class="k">Mean</span><span class="v">%.2f mm</span><span class="w">FLAME alone %.2f mm</span></div>
      <div><span class="k">95th percentile</span><span class="v">%.2f mm</span><span class="w">FLAME alone %.2f mm</span></div>
      <div><span class="k">Heads within 0.75 mm</span><span class="v">%d of 57</span><span class="w">FLAME alone %d of 57</span></div>
    </div>
    <p>The three worst heads are %s, in millimetres.</p>
''' % (key, title, cls, verdict, intro, a[0], b[0], a[1], b[1], a[2], b[2], worst))
    if props:
        out.append('    <ul class="plain">\n      ' + '\n      '.join(pro(k, k) for k in props) + '\n    </ul>\n')
    lst = lambda h, items: '      <div><h3>%s</h3><ul>\n        ' % h + '\n        '.join('<li>%s</li>' % i for i in items) + '\n      </ul></div>'
    out.append('    <div class="cols">\n' + '\n'.join([lst('What is done', done), lst('What we still dislike', dislike), lst('Next', nxt)]) + '\n    </div>\n')
    if pic:
        pnum += 1
        out.append('''    <figure>
      <img src="{{%s}}" alt="The %s cut from the feature sheet, with KOTOR's head on the left and the FLAME fit on the right.">
      <figcaption>Picture %d. %s, with KOTOR's head on the left and our FLAME fit on the right. This sheet was made before the eye opening controls existed, so the eyes show the old fixed opening.</figcaption>
    </figure>
''' % (pic, title.lower(), pnum, PIC[pic][0] if pic in ('pic_jaw',) else (PIC[pic][1] + '\'s ' + title.lower())))
    out.append('  </section>\n')
out.append('  <!--/ZONES-->\n')
sect = ''.join(out)
t = open('template.html').read()
if '<!--ZONES-->' in t:
    t = re.sub(r'  <!--ZONES-->.*?<!--/ZONES-->\n', lambda m: sect, t, flags=re.S)
else:
    t = t.replace('  <section class="step" id="targets">', sect + '\n  <section class="step" id="targets">', 1)
open('template.html', 'w').write(t)
pics = json.load(open('zone_pics.json'))
f = t.replace('{{blink}}', open('blink.b64').read().strip())
for k, v in pics.items(): f = f.replace('{{%s}}' % k, v)
assert '{{' not in f
open('likeness-hq.html', 'w').write(f)
print(len(f))
