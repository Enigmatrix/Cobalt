using System;
using System.ComponentModel;
using System.Globalization;
using System.Runtime.CompilerServices;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Data;
using System.Windows.Input;
using System.Windows.Media;
using System.Windows.Media.Imaging;

namespace Cobalt.Views.Controls
{
    /// <summary>
    ///     Interaction logic for ColorPicker.xaml
    /// </summary>
    public partial class ColorPicker
    {


        public ColorPicker()
        {
            InitializeComponent();
            Loaded += Load;
        }

        private void Load(object sender, RoutedEventArgs e)
        {
            ColorInt = ColorInternal.FromColor(((ColorPicker)sender).Color);
        }

        public Color Color
        {
            get => (Color)GetValue(ColorProperty);
            set => SetValue(ColorProperty, value);
        }

        public static readonly DependencyProperty ColorProperty =
            DependencyProperty.Register("Color", typeof(Color), typeof(ColorPicker), new PropertyMetadata(Colors.Black));



        public ColorInternal ColorInt
        {
            get => (ColorInternal)GetValue(ColorIntProperty);
            set => SetValue(ColorIntProperty, value);
        }

        public static readonly DependencyProperty ColorIntProperty =
            DependencyProperty.Register("ColorInt", typeof(ColorInternal), typeof(ColorPicker), new PropertyMetadata(null, ColorInternalChanged));

        private static void ColorInternalChanged(DependencyObject d, DependencyPropertyChangedEventArgs e)
        {
            var picker = (ColorPicker) d;
            void SetColor(object sender, PropertyChangedEventArgs propertyChangedEventArgs)
            {
                picker.ColorInt = ((ColorInternal) sender).Clone();
            }

            var oldCol = (ColorInternal) e.OldValue;
            if (oldCol != null)
                oldCol.PropertyChanged -= SetColor;
            var newCol = (ColorInternal) e.NewValue ?? ColorInternal.Black.Clone();
            newCol.PropertyChanged += SetColor;
            picker.Color = newCol.ToColor();
        }


        private BitmapSource GetBitmap()
        {
            var imgBrush = (ImageBrush) ColorSpectrum.Fill;
            var source = (BitmapSource) imgBrush.ImageSource;
            if (source.Format != PixelFormats.Bgra32)
                source = new FormatConvertedBitmap(source, PixelFormats.Bgra32, null, 0);
            return source;
        }

        public class ColorInternal : INotifyPropertyChanged
        {
            private byte _a;
            private byte _g;
            private byte _b;
            private byte _r;

            public byte R
            {
                get => _r;
                set { _r = value; OnPropertyChanged(); }
            }

            public byte B
            {
                get => _b;
                set { _b = value; OnPropertyChanged(); }
            }

            public byte G
            {
                get => _g;
                set { _g = value; OnPropertyChanged(); }
            }

            public byte A
            {
                get => _a;
                set { _a = value; OnPropertyChanged(); }
            }

            public static ColorInternal Black => FromArgb(255, 0, 0, 0);

            public event PropertyChangedEventHandler PropertyChanged;

            protected virtual void OnPropertyChanged([CallerMemberName] string propertyName = null)
            {
                PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
            }

            public Color ToColor()
            {
                return Color.FromArgb(A, R, G, B);
            }

            public static ColorInternal FromArgb(byte a, byte r, byte g, byte b)
            {
                return new ColorInternal {A = a, B = b, R = r, G = g};
            }

            public ColorInternal Clone()
            {
                return FromArgb(A, R, G, B);
            }

            public static ColorInternal FromColor(Color c)
            {
                return FromArgb(c.A, c.R, c.G, c.B);
            }
        }

        private void ColorSpectrumMouseDown(object sender, MouseButtonEventArgs e)
        {
            var point = e.GetPosition(ColorSpectrum);
            var source = GetBitmap();
            var p = new byte[] {0, 0, 0, 0};
            var x = point.X / ColorSpectrum.ActualWidth * source.PixelWidth;
            var y = point.Y / ColorSpectrum.ActualHeight * source.PixelHeight;
            source.CopyPixels(new Int32Rect((int)x, (int)y, 1, 1), p, 4, 0);

            ColorInt = ColorInternal.FromArgb(p[3], p[2], p[1], p[0]);
        }
    }
}