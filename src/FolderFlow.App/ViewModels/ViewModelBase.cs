using CommunityToolkit.Mvvm.ComponentModel;
using FolderFlow.Application.Interfaces;
using Microsoft.Extensions.DependencyInjection;

namespace FolderFlow.App.ViewModels;

public abstract partial class ViewModelBase : ObservableObject
{
    public ILocalizationService L => App.Services?.GetRequiredService<ILocalizationService>()!;
}
