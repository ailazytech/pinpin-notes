param([string]$Src = "$PSScriptRoot\deco-src", [string]$Out = "$PSScriptRoot\deco")
# Chroma-key the generated decoration images, crop, resize, rotate, save as transparent PNG.
Add-Type -AssemblyName System.Drawing
Add-Type -ReferencedAssemblies System.Drawing @"
using System; using System.Drawing; using System.Drawing.Imaging; using System.Drawing.Drawing2D;
public static class Deco {
  // key: 'g' = green screen, 'm' = magenta screen
  public static Bitmap Key(Bitmap src, char key) {
    var bmp = new Bitmap(src.Width, src.Height, PixelFormat.Format32bppArgb);
    using (var g = Graphics.FromImage(bmp)) g.DrawImage(src, 0, 0, src.Width, src.Height);
    var d = bmp.LockBits(new Rectangle(0,0,bmp.Width,bmp.Height), ImageLockMode.ReadWrite, PixelFormat.Format32bppArgb);
    int n = d.Stride * d.Height; var px = new byte[n];
    System.Runtime.InteropServices.Marshal.Copy(d.Scan0, px, 0, n);
    for (int i = 0; i < n; i += 4) {
      int b = px[i], gg = px[i+1], r = px[i+2];
      int screen = key == 'g' ? gg - Math.Max(r, b) : Math.Min(r, b) - gg;
      double a = 1.0 - Math.Min(1.0, Math.Max(0.0, (screen - 30) / 110.0));
      if (a < 1.0) {
        // despill: pull the screen colour out of the edge pixel
        if (key == 'g') gg = Math.Min(gg, Math.Max(r, b));
        else { int ex = Math.Max(0, Math.Min(r, b) - gg); r -= ex; b -= ex; }
      }
      px[i] = (byte)b; px[i+1] = (byte)gg; px[i+2] = (byte)r; px[i+3] = (byte)Math.Round(a * 255);
    }
    System.Runtime.InteropServices.Marshal.Copy(px, 0, d.Scan0, n);
    bmp.UnlockBits(d);
    return bmp;
  }
  public static Rectangle Bounds(Bitmap bmp) {
    var d = bmp.LockBits(new Rectangle(0,0,bmp.Width,bmp.Height), ImageLockMode.ReadOnly, PixelFormat.Format32bppArgb);
    int n = d.Stride * d.Height; var px = new byte[n];
    System.Runtime.InteropServices.Marshal.Copy(d.Scan0, px, 0, n); bmp.UnlockBits(d);
    int x0 = bmp.Width, y0 = bmp.Height, x1 = -1, y1 = -1;
    for (int y = 0; y < bmp.Height; y++) for (int x = 0; x < bmp.Width; x++)
      if (px[y * d.Stride + x * 4 + 3] > 12) { if (x < x0) x0 = x; if (x > x1) x1 = x; if (y < y0) y0 = y; if (y > y1) y1 = y; }
    return Rectangle.FromLTRB(x0, y0, x1 + 1, y1 + 1);
  }
  // crop to content, rotate by deg, fit inside maxW x maxH
  public static Bitmap Fit(Bitmap keyed, float deg, int maxW, int maxH) {
    var bb = Bounds(keyed);
    double rad = deg * Math.PI / 180, c = Math.Abs(Math.Cos(rad)), s = Math.Abs(Math.Sin(rad));
    double rw = bb.Width * c + bb.Height * s, rh = bb.Width * s + bb.Height * c;
    double k = Math.Min(maxW / rw, maxH / rh);
    int w = (int)Math.Ceiling(rw * k) + 2, h = (int)Math.Ceiling(rh * k) + 2;
    var outp = new Bitmap(w, h, PixelFormat.Format32bppArgb);
    using (var g = Graphics.FromImage(outp)) {
      g.InterpolationMode = InterpolationMode.HighQualityBicubic;
      g.PixelOffsetMode = PixelOffsetMode.HighQuality;
      g.SmoothingMode = SmoothingMode.HighQuality;
      g.TranslateTransform(w / 2f, h / 2f);
      g.RotateTransform(deg);
      g.ScaleTransform((float)k, (float)k);
      g.DrawImage(keyed, new RectangleF(-bb.Width / 2f, -bb.Height / 2f, bb.Width, bb.Height), bb, GraphicsUnit.Pixel);
    }
    return outp;
  }
  public static void Process(string inPath, string outPath, char key, float deg, int maxW, int maxH) {
    using (var src = new Bitmap(inPath))
    using (var keyed = Key(src, key))
    using (var fit = Fit(keyed, deg, maxW, maxH)) {
      fit.Save(outPath, ImageFormat.Png);
      Console.WriteLine("{0} {1}x{2}", System.IO.Path.GetFileName(outPath), fit.Width, fit.Height);
    }
  }
}
"@
New-Item -ItemType Directory -Force $Out | Out-Null
# name, source, key, rotation, max width, max height  (sizes are 2x the logical size)
$jobs = @(
  @('pin-red',    'pin-red.png',   'g', 0,   96, 96),
  @('pin-teal',   'pin-teal.png',  'm', 0,   96, 96),
  @('tape-pink',  'tape-pink.png', 'g', -4,  260, 90),
  @('tape-blue',  'tape-blue.png', 'm', 3,   260, 90),
  @('clip',       'clip.png',      'g', 8,   70, 190),
  @('star',       'star.png',      'g', 10,  110, 110)
)
foreach ($j in $jobs) {
  [Deco]::Process((Join-Path $Src $j[1]), (Join-Path $Out ($j[0] + '.png')), [char]$j[2], [float]$j[3], [int]$j[4], [int]$j[5])
}
