using System.ComponentModel;
using System.Runtime.CompilerServices;
using Caliburn.Micro;

namespace Cobalt.Common.UI
{
    public interface IViewModel : INotifyPropertyChanged
    {
    }

    public class NotifyPropertyChanged : Screen
    {
        protected void Set<T>(ref T field, T value, [CallerMemberName] string prop = null)
        {
            field = value;
            OnPropertyChanged(new PropertyChangedEventArgs(prop));
        }
    }

    public class ViewModelBase : NotifyPropertyChanged, IViewModel
    {

    }
}