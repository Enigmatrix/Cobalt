using System;
using System.Globalization;
using System.IO;
using Avalonia.Data.Converters;
using Avalonia.Media.Imaging;

namespace Cobalt.Converters;

/// <summary>
///     Converts from bytes to Avalonia <see cref="Bitmap" />
/// </summary>
public class BytesToImageConverter : IValueConverter
{
    public object? Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        var bytes = (byte[]?)value;
        if (bytes == null) return null;
        var ms = new MemoryStream(bytes);
        return new Bitmap(ms);
    }

    public object? ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        throw new NotImplementedException();
    }
}