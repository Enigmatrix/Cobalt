using System;
using System.Collections.ObjectModel;
using System.Reactive.Linq;
using Cobalt.Common.Data;
using Cobalt.Common.Data.Repository;
using Cobalt.Common.IoC;
using Cobalt.Common.UI.ViewModels;

namespace Cobalt.ViewModels.Pages
{
    public class TagsPageViewModel : PageViewModel
    {
        private ObservableCollection<TagViewModel> _tags;

        public ObservableCollection<TagViewModel> Tags
        {
            get => _tags;
            set => Set(ref _tags, value);
        }

        public TagsPageViewModel(IResourceScope scope, IDbRepository repo) : base(scope)
        {
            Repository = repo;
        }

        public IDbRepository Repository { get; set; }


        protected override void OnActivate(IResourceScope res)
        {
            Tags = new ObservableCollection<TagViewModel>();
            Repository.GetTags()
                .Select(x => 
                    new TagViewModel(x))
                .Subscribe(x => Tags.Add(x));
        }

        protected override void OnDeactivate(bool close, IResourceScope resources)
        {
            Tags = null;
        }

        public void SelectTag(TagViewModel tag)
        {
            tag.TaggedApps = new ObservableCollection<AppViewModel>();
            Repository.GetAppsWithTag((Tag) tag.Entity)
                .Select(x => new AppViewModel(x))
                .Subscribe(x => tag.TaggedApps.Add(x));
        }

        public void AddTag(string tagName)
        {
            var tag = new Tag {Name = tagName};
            Repository.AddTag(tag);
            Tags.Add(new TagViewModel(tag));
        }

        public void DeleteTag(TagViewModel tag)
        {
            Repository.RemoveTag((Tag)tag.Entity);
            Tags.Remove(tag);
        }

        public void AddTagToApp(TagViewModel tag, AppViewModel app)
        {
            Repository.AddTagToApp((Tag)tag.Entity, (App)app.Entity);
            tag.TaggedApps.Add(app);
        }

        public void RemoveTagFromApp(TagViewModel tag, AppViewModel app)
        {
            Repository.RemoveTagFromApp((Tag)tag.Entity, (App)app.Entity);
            tag.TaggedApps.Remove(app);
        }
    }
}