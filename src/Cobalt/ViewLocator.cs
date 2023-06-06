using System;
using Avalonia.Controls;
using Avalonia.Controls.Templates;
using Cobalt.Common.ViewModels;

namespace Cobalt;

public class ViewLocator : IDataTemplate
{
    public Control Build(object? data)
    {
        if (data is null)
            return new TextBlock { Text = "Null DataContext" };

        var name = data.GetType().FullName!.Replace("Common.ViewModel", "View").Replace("ViewModel", "View");
        var type = Type.GetType(name);

        if (type != null) return (Control)Activator.CreateInstance(type)!;

        return new TextBlock { Text = "Not Found: " + name };
    }

    public bool Match(object? data)
    {
        return data is ViewModelBase;
    }
}