using System.ComponentModel;
using System.Runtime.CompilerServices;
using Caliburn.Micro;

namespace Cobalt.Common.UI
{
    public interface IViewModel : INotifyPropertyChanged
    {
    }

    public class ViewModelBase : Screen
    {
        protected void Set<T>(ref T field, T value, [CallerMemberName] string prop = null)
        {
            field = value;
            OnPropertyChanged(new PropertyChangedEventArgs(prop));
        }
    }
}