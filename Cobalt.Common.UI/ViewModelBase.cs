using System;
using System.ComponentModel;
using System.Runtime.CompilerServices;
using Caliburn.Micro;

namespace Cobalt.Common.UI
{
    public interface IViewModel : INotifyPropertyChanged, IDisposable
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

    public abstract class ViewModelBase : NotifyPropertyChanged, IViewModel
    {
        public virtual void Dispose()
        {
            OnDeactivate(true);
        }
    }
}