using DataBase.Entity;
using Riok.Mapperly.Abstractions;

namespace DtoMappers.Mappers;

[Mapper(EnumMappingStrategy = EnumMappingStrategy.ByName)]
public partial class GmapsUserMapper
{
    [MapperIgnoreSource(nameof(GmapsUser.LatestPostedReviewId))]
    public static partial GmapsUserDto GmapsUserToDto(GmapsUser gmapsUser);

    [MapProperty(nameof(GmapsUserDto.LatestPostedReview.Id), nameof(GmapsUser.LatestPostedReviewId))]
    [MapperIgnoreTarget(nameof(GmapsUser.LatestPostedReview))]
    public static partial GmapsUser GmapsUserDtoToEntity(GmapsUserDto gmapsUserDto);
}
