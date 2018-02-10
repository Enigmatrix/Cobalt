using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace Cobalt.Common.Util
{
    public static class DateTimeUtils
    {
        public static DateTime StartOfWeek(this DateTime d) => d.AddDays(-(int) d.DayOfWeek);
        public static DateTime EndOfWeek(this DateTime d) => d.StartOfWeek().AddDays(7);
        public static DateTime StartOfMonth(this DateTime d) => d.AddDays(1 - d.Day);
        public static DateTime EndOfMonth(this DateTime d) => d.StartOfMonth().AddMonths(1);
    }
}
