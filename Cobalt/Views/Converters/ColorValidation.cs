using System;
using System.Collections.Generic;
using System.Globalization;
using System.Linq;
using System.Linq.Expressions;
using System.Text;
using System.Threading.Tasks;
using System.Windows.Controls;
using System.Windows.Media;

namespace Cobalt.Views.Converters
{
    public class ColorValidation : ValidationRule
    {
        //TODO try to get no error message at all (currently theres an empty black box)
        public override ValidationResult Validate(object value, CultureInfo cultureInfo)
        {
            var colString = (string) value;
            try
            {
                var color = ColorConverter.ConvertFromString(colString);
                if (color == null)
                    return new ValidationResult(false, "");
            }
            catch (Exception)
            {
                return new ValidationResult(false, "");
            }

            return ValidationResult.ValidResult;
        }
    }
}
