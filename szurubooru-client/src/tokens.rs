//! Types that represent various Query tokens for the API endpoints that support them. Be
//! warned that the types here help with the Type safety for the Tag names only. It does
//! not guarantee that a given API endpoint will support the given tag.

use std::fmt::Display;
use strum_macros::AsRefStr;
use urlencoding::encode;

/// A named token such as `foo:bar`
pub trait NamedToken: AsRef<str> {}

/// A type of token used for sorting. E.g: `sort:random`
pub trait SortableToken: AsRef<str> {}

/// Special tokens such as `liked` posts or `tumbleweed` that
/// don't fit into a query token or sort token
pub trait SpecialToken: AsRef<str> {}

/// Supports types that can be converted to a Query string
pub trait ToQueryString {
    /// Convert `&self` into a HTML query string
    fn to_query_string(&self) -> String;
}

/// A query token using for searching posts, tags and pools
#[derive(Debug)]
pub struct QueryToken {
    /// The key for this token. For `foo:bar` this would be `foo`
    pub key: String,
    /// The value for this token. For `foo:bar` this would be `bar`
    pub value: String,
}

impl QueryToken {
    ///
    /// Construct a named token for a search query. Final results takes the form of
    /// `key:value`. Values containing `:` and `-` are automatically escaped.
    ///
    /// `key` can either be one of the existing [NamedToken] types for convenience, or anything
    /// that implements [`AsRef<str>`] for custom tokens.
    ///
    /// ```no_run
    /// # use szurubooru_client::SzurubooruClient;
    /// # let client = SzurubooruClient::new_with_token("http://foo", "user", "pwd", true).unwrap();
    /// // let client = SzurubooruClient::new(...)
    /// use szurubooru_client::tokens::{PostNamedToken, QueryToken};
    /// // Find all posts with at least one comment...
    /// let qt = QueryToken::token(PostNamedToken::CommentCount, "0..");
    /// // ...with a positive score.
    /// let custom = QueryToken::token("score", "0..");
    /// client.request().list_posts(Some(&vec![qt, custom]));
    /// ```
    pub fn token(key: impl AsRef<str>, value: impl AsRef<str>) -> Self {
        let escaped = value.as_ref().replace(":", "\\:").replace("-", "\\-");
        Self {
            key: key.as_ref().to_string(),
            value: escaped,
        }
    }

    ///
    /// Constructs a token for sorting purposes. Final results take the form of
    /// `sort:value`.
    ///
    /// `value` can either be one of the existing [SortableToken] types for convenience or any type
    /// that implements [`AsRef<str>`]
    ///
    /// ```no_run
    /// # use szurubooru_client::SzurubooruClient;
    /// # let client = SzurubooruClient::new_with_token("http://foo", "user", "pwd", true).unwrap();
    /// // let client = SzurubooruClient::new(...)
    /// use szurubooru_client::tokens::{PostSortToken, QueryToken};
    /// // Sort posts at random
    /// let sort_token = QueryToken::sort(PostSortToken::Random);
    /// client.request().list_posts(Some(&vec![sort_token]));
    /// ```
    pub fn sort(value: impl AsRef<str>) -> Self {
        Self {
            key: "sort".to_string(),
            value: value.as_ref().to_string(),
        }
    }

    ///
    /// Constructs a new anonymous token. These are resource specific, e.g for [crate::models::PostResource] it's
    /// the same as [PostNamedToken::Tag].
    ///
    /// Keys containing `:` and `-` are automatically escaped.
    /// ```no_run
    /// # use szurubooru_client::SzurubooruClient;
    /// # let client = SzurubooruClient::new_with_token("http://foo", "user", "pwd", true).unwrap();
    /// // let client = SzurubooruClient::new(...)
    /// use szurubooru_client::tokens::QueryToken;
    /// // Fetch all posts containing the tag "re:zero"
    /// // Tag will be escaped as "re\:zero"
    /// let re_zero = QueryToken::anonymous("re:zero");
    /// client.request().list_posts(Some(&vec![re_zero]));
    /// ```
    pub fn anonymous(key: impl AsRef<str>) -> Self {
        let escaped = key.as_ref().replace(":", "\\:").replace("-", "\\-");
        Self {
            key: escaped,
            value: "".to_string(),
        }
    }

    ///
    /// Constructs a new special token. Some resource types (see [PostSpecialToken]) support
    /// special tokens. This is a convenience function for `QueryToken::anonymous` that provides
    /// type-safe construction of a QueryToken.
    /// ```no_run
    /// # use szurubooru_client::SzurubooruClient;
    /// use szurubooru_client::tokens::{PostSpecialToken, QueryToken};
    /// # let client = SzurubooruClient::new_with_token("http://foo", "user", "pwd", true).unwrap();
    /// // let client = SzurubooruClient::new(...)
    /// // Return posts liked by the current authenticated user
    /// let liked_posts = QueryToken::special(PostSpecialToken::Liked);
    /// client.request().list_posts(Some(&vec![liked_posts]));
    /// ```
    pub fn special(key: impl SpecialToken) -> Self {
        QueryToken::anonymous(key)
    }

    ///
    /// Negate the existing token. Include becomes Exclude and vice versa.
    ///
    /// E.g: `konosuba` becomes `-konosuba`
    ///
    pub fn negate(&self) -> Self {
        let negated_key = if self.key.starts_with("-") {
            self.key[1..].to_string()
        } else {
            format!("-{}", self.key)
        };

        Self {
            key: negated_key,
            value: self.value.clone(),
        }
    }
}

impl Display for QueryToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let suffix = if !self.value.is_empty() {
            format!(":{}", &self.value)
        } else {
            "".to_string()
        };
        write!(f, "{}{}", &self.key, suffix)
    }
}

impl ToQueryString for Vec<QueryToken> {
    fn to_query_string(&self) -> String {
        let query_vec: Vec<String> = self.iter().map(|qv| qv.to_string()).collect();
        query_vec.join(" ")
    }
}

#[derive(Debug, AsRefStr)]
#[strum(serialize_all = "kebab-case")]
/// Type-safe named query tokens for use with [list_tags](crate::SzurubooruRequest::list_tags)
pub enum TagNamedToken {
    /// having given name (accepts wildcards)
    Name,
    /// having given category (accepts wildcards)
    Category,
    /// created at given date
    CreationDate,
    /// edited at given date
    LastEditDate,
    /// alias of [TagNamedToken::LastEditTime]
    LastEditTime,
    /// alias of [TagNamedToken::LastEditTime]
    EditDate,
    /// alias of [TagNamedToken::LastEditTime]
    EditTime,
    /// used in given number of posts
    Usages,
    /// alias of [TagNamedToken::Usages]
    UsageCount,
    /// alias of [TagNamedToken::Usages]
    PostCount,
    /// with given number of suggestions
    SuggestionCount,
    /// with given number of implications
    ImplicationCount,
}
impl NamedToken for TagNamedToken {}

#[derive(Debug, AsRefStr)]
#[strum(serialize_all = "kebab-case")]
/// Type-safe sort query tokens for use with [list_tags](crate::SzurubooruRequest::list_tags)
pub enum TagSortToken {
    /// as random as it can get
    Random,
    /// A to Z
    Name,
    /// category (A to Z)
    Category,
    /// recently created first
    CreationDate,
    /// alias of [TagSortToken::CreationDate]
    CreationTime,
    /// recently edited first
    LastEditDate,
    /// alias of [TagSortToken::CreationTime]
    LastEditTime,
    /// alias of [TagSortToken::CreationTime]
    EditDate,
    /// alias of [TagSortToken::CreationTime]
    EditTime,
    /// used in most posts first
    Usages,
    /// alias of [TagSortToken::Usages]
    UsageCount,
    /// alias of [TagSortToken::Usages]
    PostCount,
    /// with most suggestions first
    SuggestionCount,
    /// with most implications first
    ImplicationCount,
}
impl SortableToken for TagSortToken {}

#[derive(Debug, AsRefStr)]
#[strum(serialize_all = "kebab-case")]
/// Type-safe named query tokens for use with [list_posts](crate::SzurubooruRequest::list_posts)
pub enum PostNamedToken {
    /// having given post number
    Id,
    /// having given tag (accepts wildcards)
    Tag,
    /// having given score
    Score,
    /// uploaded by given user (accepts wildcards)
    Uploader,
    /// alias of [PostNamedToken::Uploader]
    Upload,
    /// alias of [PostNamedToken::Uploader]
    Submit,
    /// commented by given user (accepts wildcards)
    Comment,
    /// favorited by given user (accepts wildcards)
    Fav,
    /// belonging to the pool with the given ID
    Pool,
    /// having given number of tags
    TagCount,
    /// having given number of comments
    CommentCount,
    /// favorited by given number of users
    FavCount,
    /// having given number of annotations
    NoteCount,
    /// having given note text (accepts wildcards)
    NoteText,
    /// having given number of relations
    RelationCount,
    /// having been featured given number of times
    FeatureCount,
    /// given type of posts. `value` can be either `image`, `animation` (or `animated` or `anim`),
    /// `flash` (or `swf`) or `video` (or `webm`). Use [models::PostType] for type-safe values
    Type,
    /// having given SHA1 checksum
    ContentChecksum,
    /// having given file size (in bytes)
    FileSize,
    /// having given image width (where applicable)
    ImageWidth,
    /// having given image height (where applicable)
    ImageHeight,
    /// having given number of pixels (image width * image height)
    ImageArea,
    /// having given aspect ratio (image width / image height)
    ImageAspectRatio,
    /// alias of [PostNamedToken::ImageAspectRatio]
    ImageAr,
    /// alias of [PostNamedToken::ImageWidth]
    Width,
    /// alias of [PostNamedToken::ImageHeight]
    Height,
    /// alias of [PostNamedToken::ImageAspectRatio]
    Ar,
    /// alias of [PostNamedToken::ImageAspectRatio]
    AspectRatio,
    /// posted at given date
    CreationDate,
    /// alias of [PostNamedToken::CreationDate]
    CreationTime,
    /// alias of [PostNamedToken::CreationDate]
    Date,
    /// alias of [PostNamedToken::CreationDate]
    Time,
    /// edited at given date
    LastEditDate,
    /// alias of [PostNamedToken::LastEditDate]
    LastEditTime,
    /// alias of [PostNamedToken::LastEditDate]
    EditDate,
    /// alias of [PostNamedToken::LastEditDate]
    EditTime,
    /// commented at given date
    CommentDate,
    /// alias of [PostNamedToken::CommentDate]
    CommentTime,
    /// last favorited at given time
    FavDate,
    /// alias of [PostNamedToken::FavDate]
    FavTime,
    /// featured at given date
    FeatureDate,
    /// alias of [PostNamedToken::FeatureDate]
    FeatureTime,
    /// having given safety. <value> can be either `safe`, `sketchy` (or `questionable`) or `unsafe`
    /// Use [models::PostSafety] for the type-safe version
    Safety,
    /// alias of [PostNamedToken::Safety]
    Rating,
}
impl NamedToken for PostNamedToken {}

#[derive(Debug, AsRefStr)]
#[strum(serialize_all = "kebab-case")]
/// Type-safe sort query tokens for use with [list_posts](crate::SzurubooruRequest::list_posts)
pub enum PostSortToken {
    /// as random as it can get
    Random,
    /// highest to lowest post number
    Id,
    /// highest scored
    Score,
    /// with most tags
    TagCount,
    /// most commented first
    CommentCount,
    /// loved by most
    FavCount,
    /// with most annotations
    NoteCount,
    /// with most relations
    RelationCount,
    /// most often featured
    FeatureCount,
    /// largest files first
    FileSize,
    /// widest images first
    ImageWidth,
    /// tallest images first
    ImageHeight,
    /// largest images first
    ImageArea,
    /// alias of [PostSortToken::ImageWidth]
    Width,
    /// alias of [PostSortToken::ImageHeight]
    Height,
    /// alias of [PostSortToken::ImageArea]
    Area,
    /// newest to oldest (pretty much same as id)
    CreationDate,
    /// alias of [PostSortToken::CreationDate]
    CreationTime,
    /// alias of [PostSortToken::CreationDate]
    Date,
    /// alias of [PostSortToken::CreationDate]
    Time,
    /// like [PostSortToken::CreationDate], only looks at last edit time
    LastEditDate,
    /// alias of [PostSortToken::LastEditDate]
    LastEditTime,
    /// alias of [PostSortToken::LastEditDate]
    EditDate,
    /// alias of [PostSortToken::LastEditDate]
    EditTime,
    /// recently commented by anyone
    CommentDate,
    /// alias of [PostSortToken::CommentDate]
    CommentTime,
    /// recently added to favorites by anyone
    FavDate,
    /// alias of [PostSortToken::FavDate]
    FavTime,
    /// recently featured
    FeatureDate,
    /// alias of [PostSortToken::FeatureDate]
    FeatureTime,
}
impl SortableToken for PostSortToken {}

#[derive(Debug, AsRefStr)]
#[strum(serialize_all = "kebab-case")]
/// Type-safe special query tokens for use with [list_posts](crate::SzurubooruRequest::list_posts)
pub enum PostSpecialToken {
    /// posts liked by currently logged-in user
    Liked,
    /// posts disliked by currently logged in user
    Disliked,
    /// posts added to favorites by currently logged-in user
    Fav,
    /// posts with score of 0, without comments and without favorites
    Tumbleweed,
}
impl SpecialToken for PostSpecialToken {}

#[derive(Debug, AsRefStr)]
#[strum(serialize_all = "kebab-case")]
/// Type-safe named query tokens for use with [list_pools](crate::SzurubooruRequest::list_pools)
pub enum PoolNamedToken {
    /// having given name (accepts wildcards)
    Name,
    /// having given category (accepts wildcards)
    Category,
    /// created at given date
    CreationDate,
    /// alias of [CreationDate](PoolNamedToken::CreationDate)
    CreationTime,
    /// edited at given date
    LastEditDate,
    /// alias of [LastEditDate](PoolNamedToken::LastEditDate)
    LastEditTime,
    /// alias of [LastEditDate](PoolNamedToken::LastEditDate)
    EditDate,
    /// alias of [LastEditDate](PoolNamedToken::LastEditDate)
    EditTime,
    /// used in given number of posts
    PostCount,
}
impl NamedToken for PoolNamedToken {}

#[derive(Debug, AsRefStr)]
#[strum(serialize_all = "kebab-case")]
/// Type-safe sort query tokens for use with [list_pools](crate::SzurubooruRequest::list_pools)
pub enum PoolSortToken {
    /// as random as it can get
    Random,
    /// A to Z
    Name,
    /// category (A to Z)
    Category,
    /// recently created first
    CreationDate,
    /// alias of [CreationDate](PoolSortToken::CreationDate)
    CreationTime,
    /// recently edited first
    LastEditDate,
    /// alias of [CreationDate](PoolSortToken::LastEditDate)
    LastEditTime,
    /// alias of [CreationDate](PoolSortToken::LastEditDate)
    EditDate,
    /// alias of [CreationDate](PoolSortToken::LastEditDate)
    EditTime,
    /// used in most posts first
    PostCount,
}
impl SortableToken for PoolSortToken {}

#[derive(Debug, AsRefStr)]
#[strum(serialize_all = "kebab-case")]
/// Type-safe named query tokens for use with
/// [list_comments](crate::SzurubooruRequest::list_comments)
pub enum CommentNamedToken {
    /// specific comment ID
    Id,
    /// specific post ID
    Post,
    /// created by given user (accepts wildcards)
    User,
    /// alias of user
    Author,
    /// containing given text (accepts wildcards)
    Text,
    /// created at given date
    CreationDate,
    /// alias of creation-date
    CreationTime,
    /// whose most recent edit date matches given date
    LastEditDate,
    /// alias of last-edit-date
    LastEditTime,
    /// alias of last-edit-date
    EditDate,
    /// alias of last-edit-date
    EditTime,
}
impl NamedToken for CommentNamedToken {}

#[derive(Debug, AsRefStr)]
#[strum(serialize_all = "kebab-case")]
/// Type-safe sort query tokens for use with
/// [list_comments](crate::SzurubooruRequest::list_comments)
pub enum CommentSortToken {
    /// as random as it can get
    Random,
    /// author name, A to Z
    User,
    /// alias of user
    Author,
    /// post ID, newest to oldest
    Post,
    /// newest to oldest
    CreationDate,
    /// alias of creation-date
    CreationTime,
    /// recently edited first
    LastEditDate,
    /// alias of last-edit-date
    LastEditTime,
    /// alias of last-edit-date
    EditDate,
    /// alias of last-edit-date
    EditTime,
}
impl SortableToken for CommentSortToken {}

#[derive(Debug, AsRefStr)]
#[strum(serialize_all = "kebab-case")]
/// Type-safe named query tokens for use with [list_users](crate::SzurubooruRequest::list_users)
pub enum UserNamedToken {
    /// having given name (accepts wildcards)
    Name,
    /// registered at given date
    CreationDate,
    /// alias of [CreationDate](UserNamedToken::CreationDate)
    CreationTime,
    /// whose most recent login date matches given date
    LastLoginDate,
    /// alias of [LastLoginDate](UserNamedToken::LastLoginDate)
    LastLoginTime,
    /// alias of [LastLoginDate](UserNamedToken::LastLoginDate)
    LoginDate,
    /// alias of [LastLoginDate](UserNamedToken::LastLoginDate)
    LoginTime,
}
impl NamedToken for UserNamedToken {}

#[derive(Debug, AsRefStr)]
#[strum(serialize_all = "kebab-case")]
/// Type-safe sort query tokens for use with [list_users](crate::SzurubooruRequest::list_users)
pub enum UserSortToken {
    /// as random as it can get
    Random,
    /// A to Z
    Name,
    /// newest to oldest
    CreationDate,
    /// alias of [CreationDate](UserSortToken::CreationDate)
    CreationTime,
    /// recently active first
    LastLoginDate,
    /// alias of [LastLoginDate](UserSortToken::LastLoginDate)
    LastLoginTime,
    /// alias of [LastLoginDate](UserSortToken::LastLoginDate)
    LoginDate,
    /// alias of [LastLoginDate](UserSortToken::LastLoginDate)
    LoginTime,
}
impl SortableToken for UserNamedToken {}

#[derive(Debug, AsRefStr)]
#[strum(serialize_all = "kebab-case")]
/// Type-safe named query tokens for use with
/// [list_snapshots](crate::SzurubooruRequest::list_snapshots)
pub enum SnapshotNamedToken {
    /// involving given resource type
    Type,
    /// involving given resource id
    Id,
    /// created at given date
    Date,
    /// alias of date
    Time,
    /// modified, created, deleted or merged
    Operation,
    /// name of the user that created given snapshot (accepts wildcards)
    User,
}
impl NamedToken for SnapshotNamedToken {}

#[cfg(test)]
mod tests {
    use crate::tokens::*;

    #[test]
    fn test_query_token() {
        let qt = QueryToken::token(PostNamedToken::CommentCount, "1");
        assert_eq!(qt.to_string(), "comment-count:1");

        let qt = qt.negate();
        assert_eq!(qt.to_string(), "-comment-count:1");

        let qt = QueryToken::sort(PostSortToken::Random);
        assert_eq!(qt.to_string(), "sort:random");

        let qt = QueryToken::token(TagNamedToken::Name, "re:zero");
        assert_eq!(qt.to_string(), r#"name:re\:zero"#);

        let qt = QueryToken::special(PostSpecialToken::Liked);
        assert_eq!(qt.to_string(), "liked");

        let qt = QueryToken::anonymous("foo");
        assert_eq!(qt.to_string(), "foo");
    }

    #[test]
    fn test_vec_query() {
        let query_vec = vec![
            QueryToken::token(PostNamedToken::CommentCount, "1"),
            QueryToken::sort(PostSortToken::Random),
        ];

        assert_eq!(query_vec.to_query_string(), "comment-count:1 sort:random");
    }
}
