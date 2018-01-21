using System;

namespace Cobalt.Common.Data
{
    public class Interaction : Entity
    {
        private DateTime _timestamp = DateTime.MinValue;

        public DateTime Timestamp
        {
            get
            {
                if (_timestamp == DateTime.MinValue)
                    _timestamp = new DateTime(Id);
                return _timestamp;
            }
            set
            {
                _timestamp = value;
                Id = _timestamp.Ticks;
            }
        }
    }
}