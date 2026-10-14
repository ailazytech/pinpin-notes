param([string]$Src = "$PSScriptRoot\deco-src", [string]$Out = "$PSScriptRoot\deco")
# Split generated sheets (stickers, pins, clips, tapes) into individual transparent PNGs.
# key: chroma screen colour (g/m), or n = none (the sheet already has a transparent background); combined with any alpha the image already has.
# Objects are found as connected components; tiny specks are dropped.
Add-Type -AssemblyName System.Drawing
Add-Type -ReferencedAssemblies System.Drawing @"
using System; using System.Collections.Generic; using System.Drawing; using System.Drawing.Imaging; using System.Drawing.Drawing2D;
public static class Sheet {
  public static string Split(string inPath, string outDir, string prefix, string names, char key, int maxW, int maxH) {
    var nameList = names.Split(',');
    byte[] px; int w, h;
    using (var src = new Bitmap(inPath))
    using (var bmp = new Bitmap(src.Width, src.Height, PixelFormat.Format32bppArgb)) {
      using (var g = Graphics.FromImage(bmp)) g.DrawImage(src, 0, 0, src.Width, src.Height);
      w = bmp.Width; h = bmp.Height;
      var d = bmp.LockBits(new Rectangle(0,0,w,h), ImageLockMode.ReadOnly, PixelFormat.Format32bppArgb);
      px = new byte[w * h * 4];
      System.Runtime.InteropServices.Marshal.Copy(d.Scan0, px, 0, px.Length); bmp.UnlockBits(d);
    }
    // chroma key (multiplied with existing alpha) + despill
    for (int i = 0; i < px.Length; i += 4) {
      int b = px[i], gg = px[i+1], r = px[i+2], a0 = px[i+3];
      int screen = key == 'n' ? 0 : key == 'g' ? gg - Math.Max(r, b) : Math.Min(r, b) - gg;
      double a = 1.0 - Math.Min(1.0, Math.Max(0.0, (screen - 30) / 110.0));
      if (a < 1.0) {
        if (key == 'g') gg = Math.Min(gg, Math.Max(r, b));
        else { int ex = Math.Max(0, Math.Min(r, b) - gg); r -= ex; b -= ex; }
      }
      px[i] = (byte)b; px[i+1] = (byte)gg; px[i+2] = (byte)r; px[i+3] = (byte)Math.Round(a * a0);
    }
    // connected components on solid pixels
    var label = new int[w * h];
    var comps = new List<int[]>(); // minx,miny,maxx,maxy,area,id
    var stack = new Stack<int>();
    int next = 1;
    for (int start = 0; start < w * h; start++) {
      if (label[start] != 0 || px[start * 4 + 3] < 160) continue;
      int id = next++; int minx = w, miny = h, maxx = 0, maxy = 0, area = 0;
      label[start] = id; stack.Push(start);
      while (stack.Count > 0) {
        int p = stack.Pop(); int x = p % w, y = p / w; area++;
        if (x < minx) minx = x; if (x > maxx) maxx = x; if (y < miny) miny = y; if (y > maxy) maxy = y;
        for (int k = 0; k < 4; k++) {
          int nx = x + (k == 0 ? 1 : k == 1 ? -1 : 0), ny = y + (k == 2 ? 1 : k == 3 ? -1 : 0);
          if (nx < 0 || ny < 0 || nx >= w || ny >= h) continue;
          int q = ny * w + nx;
          if (label[q] == 0 && px[q * 4 + 3] >= 160) { label[q] = id; stack.Push(q); }
        }
      }
      comps.Add(new int[] { minx, miny, maxx, maxy, area, id });
    }
    int minArea = w * h / 400;
    var keep = comps.FindAll(c => c[4] >= minArea);
    // merge pieces that sit right next to each other (e.g. a moon and its little star)
    var owner = new Dictionary<int, int>();
    foreach (var c in comps) owner[c[5]] = c[5];
    bool merged = true;
    while (merged) {
      merged = false;
      for (int i = 0; i < keep.Count && !merged; i++) for (int j = i + 1; j < keep.Count && !merged; j++) {
        var a = keep[i]; var b = keep[j];
        int gx = Math.Max(0, Math.Max(a[0], b[0]) - Math.Min(a[2], b[2]));
        int gy = Math.Max(0, Math.Max(a[1], b[1]) - Math.Min(a[3], b[3]));
        if (gx < 14 && gy < 14) {
          a[0] = Math.Min(a[0], b[0]); a[1] = Math.Min(a[1], b[1]); a[2] = Math.Max(a[2], b[2]); a[3] = Math.Max(a[3], b[3]); a[4] += b[4];
          foreach (var k in new List<int>(owner.Keys)) if (owner[k] == b[5]) owner[k] = a[5];
          keep.RemoveAt(j); merged = true;
        }
      }
    }
    for (int i = 0; i < label.Length; i++) if (label[i] != 0) label[i] = owner[label[i]];
    // reading order: rows, then columns
    keep.Sort((a, b) => ((a[1] + a[3]) / 2).CompareTo((b[1] + b[3]) / 2));
    var rows = new List<List<int[]>>();
    foreach (var c in keep) {
      int cy = (c[1] + c[3]) / 2;
      var row = rows.Count > 0 ? rows[rows.Count - 1] : null;
      if (row != null && Math.Abs(cy - (row[0][1] + row[0][3]) / 2) < (row[0][3] - row[0][1]) / 2) row.Add(c);
      else rows.Add(new List<int[]> { c });
    }
    var ordered = new List<int[]>();
    foreach (var row in rows) { row.Sort((a, b) => a[0].CompareTo(b[0])); ordered.AddRange(row); }
    var report = new System.Text.StringBuilder();
    for (int n = 0; n < ordered.Count; n++) {
      var c = ordered[n]; int id = c[5];
      int pad = 6;
      int x0 = Math.Max(0, c[0] - pad), y0 = Math.Max(0, c[1] - pad), x1 = Math.Min(w - 1, c[2] + pad), y1 = Math.Min(h - 1, c[3] + pad);
      int cw = x1 - x0 + 1, ch = y1 - y0 + 1;
      // keep the component plus a 3px halo (anti-aliased edge), drop everything else (specks)
      var crop = new Bitmap(cw, ch, PixelFormat.Format32bppArgb);
      var cd = crop.LockBits(new Rectangle(0,0,cw,ch), ImageLockMode.WriteOnly, PixelFormat.Format32bppArgb);
      var cp = new byte[cw * ch * 4];
      for (int y = 0; y < ch; y++) for (int x = 0; x < cw; x++) {
        int sx = x0 + x, sy = y0 + y; bool near = false;
        for (int dy = -3; dy <= 3 && !near; dy++) for (int dx = -3; dx <= 3 && !near; dx++) {
          int qx = sx + dx, qy = sy + dy;
          if (qx >= 0 && qy >= 0 && qx < w && qy < h && label[qy * w + qx] == id) near = true;
        }
        if (!near) continue;
        int si = (sy * w + sx) * 4, di = (y * cw + x) * 4;
        cp[di] = px[si]; cp[di+1] = px[si+1]; cp[di+2] = px[si+2]; cp[di+3] = px[si+3];
      }
      System.Runtime.InteropServices.Marshal.Copy(cp, 0, cd.Scan0, cp.Length); crop.UnlockBits(cd);
      double k = Math.Min(1.0, Math.Min(maxW / (double)cw, maxH / (double)ch));
      int ow = Math.Max(1, (int)Math.Round(cw * k)), oh = Math.Max(1, (int)Math.Round(ch * k));
      using (var outp = new Bitmap(ow, oh, PixelFormat.Format32bppArgb)) {
        using (var g = Graphics.FromImage(outp)) {
          g.InterpolationMode = InterpolationMode.HighQualityBicubic; g.PixelOffsetMode = PixelOffsetMode.HighQuality;
          g.DrawImage(crop, new Rectangle(0, 0, ow, oh));
        }
        string name = n < nameList.Length ? nameList[n] : ("x" + n);
        outp.Save(System.IO.Path.Combine(outDir, prefix + "-" + name + ".png"), ImageFormat.Png);
        report.AppendFormat("{0}-{1} {2}x{3}\n", prefix, name, ow, oh);
      }
      crop.Dispose();
    }
    report.AppendFormat("({0} objects, {1} specks dropped)\n", ordered.Count, comps.Count - keep.Count);
    return report.ToString();
  }
}
"@
New-Item -ItemType Directory -Force $Out | Out-Null
# sheet, prefix, names (reading order), key, max width, max height (2x logical size)
$jobs = @(
  @('sheet-cute.png',   'st', 'cat,shiba,boba,strawberry,donut,onigiri,sun,rainbow,apple', 'g', 150, 150),
  @('sheet-icons.png',  'st', 'heart,star,thumbsup,check,alert,fire,bolt,sparkles,coffee', 'g', 150, 150),
  @('sheet-nature.png', 'st', 'clover,cactus,monstera,sunflower,mushroom,cloud,moon,sakura,rain', 'm', 150, 150),
  @('sheet-pins.png',   'pin', 'mint,gold,silver,pearl,heart,black', 'm', 90, 90),
  @('sheet-clips.png',  'clip', 'gold,heart,mint,binder,clothespin,blue', 'm', 120, 180),
  @('sheet-tapes.png',  'tape', 'kraft,gingham,grid,dusty,stars,floral', 'm', 300, 80)
)
foreach ($j in $jobs) {
  [Sheet]::Split((Join-Path $Src $j[0]), $Out, $j[1], $j[2], [char]$j[3], [int]$j[4], [int]$j[5])
}
