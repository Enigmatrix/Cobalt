using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Linq;
using System.Runtime.CompilerServices;
using System.Text;
using System.Threading.Tasks;
using Caliburn.Micro;

namespace Cobalt.Common.UI
{
    public class ViewModelBase : Screen
    {
        public void Set<T>(ref T field, T value, [CallerMemberName]string prop = null)
        {
            field = value;
            OnPropertyChanged(new PropertyChangedEventArgs(prop));
        }
    }
}
