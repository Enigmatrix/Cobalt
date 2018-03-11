using System;

namespace Cobalt.Common.Util
{
    public static class DateTimeUtils
    {
        public static DateTime StartOfWeek(this DateTime d)
        {
            return d.AddDays(-(int) d.DayOfWeek);
        }

        public static DateTime EndOfWeek(this DateTime d)
        {
            return d.StartOfWeek().AddDays(7);
        }

        public static DateTime StartOfMonth(this DateTime d)
        {
            return d.AddDays(1 - d.Day);
        }

        public static DateTime EndOfMonth(this DateTime d)
        {
            return d.StartOfMonth().AddMonths(1);
        }

        public static DateTime Min(this DateTime d1, DateTime d2)
        {
            return d1 < d2 ? d1 : d2;
        }

        public static DateTime Max(this DateTime d1, DateTime d2)
        {
            return d1 < d2 ? d2 : d1;
        }
    }
}