using DataBase.Entity;
using Riok.Mapperly.Abstractions;

namespace DtoMappers.Mappers;

[Mapper(EnumMappingStrategy = EnumMappingStrategy.ByName)]
public partial class PostedReviewMapper
{
    public static partial PostedReviewDto PostedReviewToDto(PostedReview postedReview);

    [MapProperty(nameof(PostedReviewDto.GmapsUser.Id), nameof(PostedReview.GmapsUserId))]
    [MapperIgnoreTarget(nameof(PostedReview.GmapsUser))]
    public static partial PostedReview PostedReviewDtoToEntity(PostedReviewDto postedReviewDto);
}
