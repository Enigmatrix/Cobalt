using System.ComponentModel;
using System.Runtime.CompilerServices;
using Caliburn.Micro;

namespace Cobalt.Common.UI
{
    public class ViewModelBase : Screen
    {
        public void Set<T>(ref T field, T value, [CallerMemberName] string prop = null)
        {
            field = value;
            OnPropertyChanged(new PropertyChangedEventArgs(prop));
        }
    }
}